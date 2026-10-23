//! Windowed development entry point for the shared movement simulation. This
//! does not claim campaign/controller support and never writes campaign saves.
use super::{
    hosting::{Hosting, Status},
    invite::Invite,
    protocol::{Identity, Input, RemoteInput, Visit},
    session::{Guest, Host, Phase},
    transport::{self, Conditions},
    world::{self, Frame, Roster},
    PlayerId,
};
use crate::{assets::Assets, inventory::Stats, movement::FIXED_DT, Options};
use anyhow::{Context, Result};
use macroquad::prelude::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct Launch {
    pub host: bool,
    pub internet: bool,
    pub invite_file: Option<PathBuf>,
    pub invite: Option<Invite>,
    pub start: Option<Visit>,
    pub drive: bool,
}
pub async fn run(assets: &mut Assets, options: &Options, launch: &Launch) -> Result<()> {
    let ui = crate::ui::Ui::load(assets)?;
    show_mouse(false);
    let _ = std::fs::remove_file(crate::preferences::path("coop-live.json"));
    let mut hosting = None;
    let result = async {
        let prefs = crate::preferences::Preferences::load();
        let mut setup_input = crate::input::Input::default();
        let mut setup_selected = 0;
        let identity = Identity::current(assets.fingerprint()?);
        let (mut host, mut guest, invite) = if launch.host {
            hosting = Some(Hosting::start(crate::preferences::path("coop"),
                PathBuf::from("private/coop/helpers"), launch.internet, Conditions::default())?);
            let setup = hosting.as_ref().unwrap();
            let (peer, invite) = loop {
                if is_quit_requested() || is_key_pressed(KeyCode::Escape) { return Ok(()); }
                if let Some(ready) = setup.take_ready()? { break ready; }
                clear_background(BLACK);
                super::menu::panel(&ui, "Host co-op");
                super::menu::text(&ui, match setup.status() {
                    Status::PreparingHelper => "Preparing internet hosting...",
                    _ => "Creating your invite...",
                }, 150., 18.);
                setup_input.update(&prefs, crate::look::window_focused());
                if super::menu::buttons(&ui, &setup_input, &mut setup_selected, &["Cancel"], 280.).is_some() { return Ok(()); }
                if crate::look::window_focused() { ui.cursor(); }
                super::menu::capture();
                next_frame().await;
            };
            let visit = launch.start.clone().unwrap_or_else(|| Visit { epoch: 1, map: options.map.clone(), entry: options.entry.clone(), difficulty: options.difficulty, combat_preview: options.combat_preview });
            let session = Host::new(peer, identity, visit)?;
            if let Some(path) = &launch.invite_file {
                if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
                std::fs::write(path, invite.link())?;
            }
            (Some(session), None, Some(invite))
        } else {
            let invite = if let Some(invite) = &launch.invite { invite.clone() } else {
                Invite::parse(&std::fs::read_to_string(launch.invite_file.as_ref().context("Paste an invite link first.")?).context("The host invite is not ready.")?)?
            };
            (None, Some(Guest::new(transport::join(invite, Conditions::default())?, identity, Instant::now())?), None)
        };
        let local = if launch.host { PlayerId::Blue } else { PlayerId::Red };
        let mut visit = host.as_ref().map(|h| h.visit().clone());
        while visit.is_none() {
            if is_quit_requested() || is_key_pressed(KeyCode::Escape) { return Ok(()); }
            let session = guest.as_mut().unwrap();
            session.poll(Instant::now())?;
            visit = session.take_visit();
            clear_background(BLACK);
            super::menu::panel(&ui, "Join co-op");
            super::menu::text(&ui, "Connecting to your friend...", 150., 18.);
            setup_input.update(&prefs, crate::look::window_focused());
            if super::menu::buttons(&ui, &setup_input, &mut setup_selected, &["Cancel"], 280.).is_some() { return Ok(()); }
            if crate::look::window_focused() { ui.cursor(); }
            super::menu::capture();
            next_frame().await;
        }
        let visit = visit.unwrap();
        clear_background(BLACK);
        ui.toast("Opening Wonderland for two...", screen_height() * 0.5);
        next_frame().await;
        let mut scene = crate::render::Scene::load_for(assets, &visit.map, visit.difficulty)?;
        let mut arena = super::arena::Arena::load(assets, &scene.map, &visit, visit.combat_preview)?;
        let hud = crate::hud::Hud::load(assets)?;
        let mut pickup_art = crate::powerups::Art::load(assets)?;
        let (eye, initial_yaw) = crate::interaction::spawn(&scene.map, visit.entry.as_deref());
        let eye = options.start_at.map_or(eye, |p| p + Vec3::Z * crate::movement::EYE_HEIGHT);
        let mut stats = Stats::for_level(&visit.map, visit.entry.as_deref());
        stats.difficulty = visit.difficulty;
        let mut roster = if launch.host { Some(Roster::new(&scene.world, eye, initial_yaw, stats)?) } else { None };
        if let Some(roster) = &roster { arena.reset_characters(&roster.players); }
        if let Some(host) = &mut host { host.loaded(visit.epoch)?; }
        if let Some(guest) = &mut guest { guest.loaded(visit.epoch)?; }
        let mut display = roster.as_ref().map(|r| r.frame(visit.epoch, 0));
        let mut presented: Option<Frame> = None;
        let mut yaw = initial_yaw;
        let mut pitch = 0.;
        let mut look = crate::look::MouseLook::default();
        look.sensitivity = prefs.sensitivity;
        look.inverted = prefs.invert_mouse;
        let mut controls = crate::input::Input::default();
        let mut camera = crate::camera::Follow::default();
        let mut local_latch = RemoteInput::default();
        let mut keys = Input::default();
        let mut local_sequence = 0;
        let mut accumulator = 0_f64;
        let mut menu = false;
        let mut menu_selected = 0;
        let mut copied_at: Option<Instant> = None;
        let mut rejected_guest = String::new();
        let mut last_sent = 0;
        let mut rendered = 0;
        let mut started_playing = None;
        let mut report_at = Instant::now();
        let mut guest_send_at = Instant::now();
        let mut weapon_buttons = crate::weapon_rules::Buttons::default();
        weapon_buttons.block();
        let mut captured_midpoint = false;
        loop {
            let now = Instant::now();
            if let Some(setup) = &hosting {
                if let Status::Failed(reason) = setup.status() { anyhow::bail!("{reason}"); }
            }
            if let Some(host) = &mut host {
                // A bad/incompatible guest is disconnected by Host::poll.
                // Keep the host and its invite available for a valid rejoin.
                if let Err(error) = host.poll(now) { rejected_guest = error.to_string(); }
                if host.phase() == Phase::Playing { rejected_guest.clear(); }
            }
            if let Some(guest) = &mut guest {
                guest.poll(now)?;
                if let Some(mut frame) = guest.take_frame() {
                    if let Some(combat) = frame.combat.take() { arena.replicate(&combat)?; }
                    display = Some(frame);
                }
            }
            let playing = host.as_ref().map_or_else(|| guest.as_ref().unwrap().phase() == Phase::Playing,
                |h| h.phase() == Phase::Playing);
            if playing { started_playing.get_or_insert(now); }
            let elapsed = get_frame_time().clamp(0., 0.1);
            let focused = crate::look::window_focused();
            controls.update(&prefs, focused);
            if focused && controls.ui(KeyCode::Escape) { menu = !menu; menu_selected = 0; }
            if is_quit_requested() || menu && focused && is_key_pressed(KeyCode::Q) { break; }
            if focused && is_key_pressed(KeyCode::F8) {
                if let Some(invite) = &invite { miniquad::window::clipboard_set(&invite.link()); copied_at = Some(now); }
            }
            look.update(focused && !menu && !launch.drive && playing, &mut yaw, &mut pitch);
            let flat = vec2(yaw.cos(), yaw.sin());
            let right = vec2(yaw.sin(), -yaw.cos());
            let direction = vec3(yaw.cos() * pitch.cos(), yaw.sin() * pitch.cos(), pitch.sin());
            let input_active = focused && !menu && playing;
            let held = |key| input_active && controls.key(&prefs, key, false);
            let pressed = |key| input_active && controls.key(&prefs, key, true);
            let axes = if input_active { controls.movement() } else { Vec2::ZERO };
            let forward = axes.y + f32::from(held(KeyCode::W)) - f32::from(held(KeyCode::S));
            let sideways = axes.x + f32::from(held(KeyCode::D)) - f32::from(held(KeyCode::A));
            keys.wish = (flat * forward + right * sideways).clamp_length_max(1.);
            keys.swim = (direction * forward + right.extend(0.) * sideways).clamp_length_max(1.);
            keys.rise = f32::from(held(KeyCode::Space)) - f32::from(held(KeyCode::LeftControl));
            keys.run = input_active && prefs.run(held(KeyCode::LeftShift));
            keys.yaw = (yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            keys.pitch = pitch;
            keys.jumps += u64::from(pressed(KeyCode::Space));
            keys.uses += u64::from(pressed(KeyCode::E));
            let click = weapon_buttons.sample([
                controls.action(&prefs, "Mouse 1", false), controls.action(&prefs, "Mouse 2", false)
            ], input_active);
            keys.primary = click == Some(false);
            keys.alternate = click == Some(true);
            for i in 0..2 { keys.attack_presses[i] += u64::from(weapon_buttons.pressed[i]
                && if i == 0 { keys.primary } else { keys.alternate }); }
            if !input_active { keys.weapon = None; }
            for (i, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
                KeyCode::Key5, KeyCode::Key6, KeyCode::Key7, KeyCode::Key8, KeyCode::Key9, KeyCode::Key0].into_iter().enumerate() {
                if pressed(key) { keys.weapon = Some(i as u8); }
            }
            // Deterministic local test input still travels through the ordinary
            // session and host simulation, even while this window lacks focus.
            if launch.drive && playing {
                let age = started_playing.unwrap().elapsed().as_secs_f32();
                keys.run = false;
                keys.wish = if (2. ..4.).contains(&age) { if launch.host { Vec2::Y } else { Vec2::X } } else { Vec2::ZERO };
                keys.swim = keys.wish.extend(0.);
                if visit.combat_preview {
                    // A reproducible real-weapon encounter, not injected damage.
                    keys.wish = Vec2::ZERO;
                    keys.swim = Vec3::ZERO;
                    keys.yaw = std::f32::consts::FRAC_PI_2;
                    keys.pitch = 0.;
                    keys.weapon = Some(0);
                    keys.primary = false;
                    if let (Some(frame), Some(target)) = (&display, arena.npcs.targets().first()) {
                        let delta = target.center - frame.players[local.index()].player.eye();
                        keys.yaw = delta.y.atan2(delta.x);
                        keys.pitch = (delta.z / delta.length().max(1.)).asin().clamp(-1.56, 1.56);
                        // Give the guard time to strike the nearer Red player
                        // before both blades can interrupt its attack animation.
                        if (8. ..13.).contains(&age) {
                            keys.wish = if delta.truncate().length() > 60. {
                                delta.truncate().normalize_or_zero()
                            } else { Vec2::ZERO };
                            keys.primary = delta.length() < 95.;
                        }
                    }
                }
            }
            local_sequence += 1;
            local_latch.accept(local_sequence, keys, now)?;
            if let Some(guest) = &mut guest {
                if guest_send_at.elapsed() >= Duration::from_millis(33) && guest.controls(keys)? { guest_send_at = now; }
            }
            if let (Some(host), Some(roster)) = (&mut host, &mut roster) {
                if playing {
                    accumulator += f64::from(elapsed);
                    while accumulator >= f64::from(FIXED_DT) {
                        arena.step(roster, &scene.world, [local_latch.sample(now), host.controls(now)])?;
                        accumulator -= f64::from(FIXED_DT);
                    }
                    let frame = roster.frame(visit.epoch, host.acknowledged_input());
                    if frame.tick >= last_sent + 6 {
                        let mut network = frame.clone();
                        network.combat = Some(Box::new(arena.snapshot()));
                        if host.send_frame(network)? { last_sent = frame.tick; }
                    }
                    display = Some(frame);
                } else { accumulator = 0.; }
            }
            clear_background(BLACK);
            if let Some(frame) = &display {
                let mut visual = frame.clone();
                if !launch.host {
                    if let Some(previous) = &presented {
                        for (body, old) in visual.players.iter_mut().zip(&previous.players) {
                            if body.player.feet.distance(old.player.feet) < 128. {
                                body.player.feet = old.player.feet.lerp(body.player.feet, (elapsed * 20.).min(1.));
                            }
                        }
                    }
                }
                let watched = world::camera_player(&visual, local);
                let watched_body = &visual.players[watched.index()];
                let (camera_eye, target, show_local) = camera.update(&scene.world, watched_body.player.feet, direction, 180., elapsed);
                let view = Camera3D { position: camera_eye, target, up: Vec3::Z,
                    fovy: 60_f32.to_radians(), z_near: 2., z_far: 30000., ..Default::default() };
                set_camera(&view);
                crate::lighting::select(vec![], camera_eye, &scene.world);
                crate::render_fx::begin_view(&view, frame.tick as f32 * FIXED_DT, &scene.atmosphere, false);
                scene.draw(camera_eye, frame.tick as f32 * FIXED_DT, false, false, &[]);
                for (index, (character, body)) in arena.characters.iter_mut().zip(&visual.players).enumerate() {
                    character.power_appearance(&body.stats);
                    character.water_appearance(&body.player);
                    character.atmosphere(&scene.atmosphere, camera_eye);
                    let show_body = (index != watched.index() || show_local)
                        && camera_eye.distance(body.player.feet + Vec3::Z * 32.) > 48.
                        // Close partners can pass through one another in tight
                        // passages. Show the watched body instead of overlapping skins.
                        && (index == watched.index() || body.player.feet.distance(watched_body.player.feet) > 24.);
                    if show_body {
                        character.draw_replica(body.player.feet, frame.players[index].player.feet, false);
                    }
                    character.draw_effects(camera_eye, false, show_body);
                }
                arena.npcs.draw(camera_eye, (target - camera_eye).normalize_or_zero(), &scene.atmosphere, false);
                let personal = &frame.players[local.index()].stats;
                hud.draw_pickups(&arena.pickups, personal, camera_eye, frame.tick as f32 * FIXED_DT);
                pickup_art.draw(&arena.pickups, personal, &arena.visit_key, &scene.atmosphere, camera_eye, frame.tick as f32 * FIXED_DT);
                if let Some(loot) = personal.loot.get(&arena.visit_key) {
                    pickup_art.draw(&loot.pickups(&arena.visit_key), personal, &arena.visit_key, &scene.atmosphere, camera_eye, frame.tick as f32 * FIXED_DT);
                }
                scene.draw(camera_eye, frame.tick as f32 * FIXED_DT, false, true, &[]);
                presented = Some(visual);
                set_default_camera();
                hud.draw(personal, &arena.catalog, false, &controls, &prefs);
                ui.toast(&format!("{} Alice / Sanity {:.0}", if launch.host { "Blue" } else { "Red" }, frame.players[local.index()].stats.sanity()), 24.);
                if !frame.players[local.index()].stats.alive() {
                    let message = match frame.lives.state(local) {
                        super::rules::Life::Spectating { ticks_left } => format!("Following your partner / returning in {}", (ticks_left + 119) / 120),
                        _ => "Waiting for a safe place beside your partner...".into(),
                    };
                    ui.toast(&message, screen_height() * 0.4);
                }
                if report_at.elapsed() >= Duration::from_millis(200) {
                    let path = crate::preferences::path("coop-live.json");
                    if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
                    // Diagnostics are optional. A reader/virus scanner can
                    // briefly hold this file on Windows; never end gameplay.
                    if let Ok(bytes) = serde_json::to_vec(&serde_json::json!({ "role": if launch.host { "host" } else { "guest" },
                        "pid": std::process::id(), "playing": playing, "frame": frame, "focused": focused,
                        "camera_player": watched, "camera": camera_eye, "combat": arena.snapshot() })) {
                        let _ = std::fs::write(path, bytes);
                    }
                    report_at = now;
                }
            }
            set_default_camera();
            ui.toast(if playing { "Co-op combat preview / campaign integration in progress" }
                else if started_playing.is_some() { "Your partner left. Share the invite to rejoin." }
                else { "Waiting for the other player to finish loading..." }, screen_height() - 110.);
            ui.toast(if menu { "Esc: resume / Q: leave co-op" } else if launch.host { "Esc: menu / F8: copy invite" } else { "Esc: menu" }, screen_height() - 50.);
            // This panel is part of the current frame, not a nested waiting
            // loop. Network input, simulation and snapshots continue above it.
            if menu || !playing {
                show_mouse(false);
                super::menu::panel(&ui, if launch.host { "Hosting co-op" } else { "Co-op" });
                let message = if copied_at.is_some_and(|at| at.elapsed() < Duration::from_secs(4)) {
                    "Invite copied. Send it to your friend."
                } else if !rejected_guest.is_empty() { &rejected_guest }
                else if playing { "Your partner can keep playing while this menu is open." }
                else if started_playing.is_some() { "Your partner left. They can rejoin with the same invite." }
                else if launch.host { "Copy your invite and send it to your friend. They choose Join and paste the link." }
                else { "Waiting for both players to finish loading..." };
                super::menu::text(&ui, message, 128., 16.);
                let labels: &[&str] = if playing {
                    if launch.host { &["Resume", "Copy invite", "Leave co-op"] } else { &["Resume", "Leave co-op"] }
                } else if launch.host { &["Copy invite", "Leave co-op"] } else { &["Leave co-op"] };
                menu_selected = menu_selected.min(labels.len() - 1);
                if let Some(choice) = super::menu::buttons(&ui, &controls, &mut menu_selected, labels, 224.) {
                    match labels[choice] {
                        "Resume" => menu = false,
                        "Copy invite" => {
                            miniquad::window::clipboard_set(&invite.as_ref().unwrap().link());
                            copied_at = Some(now);
                        },
                        _ => break,
                    }
                }
                if focused { ui.cursor(); }
                super::menu::capture();
            }
            if focused && is_key_pressed(KeyCode::F12) {
                crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
            }
            rendered += u64::from(playing);
            if launch.drive && !captured_midpoint && started_playing.is_some_and(|start| start.elapsed() >= Duration::from_secs(6)) {
                crate::viewer::save_capture(&crate::preferences::path("coop-midpoint.png"))?;
                captured_midpoint = true;
            }
            if launch.drive && started_playing.is_some_and(|start| start.elapsed()
                >= Duration::from_secs(if launch.host { 12 } else { 10 }
                    + if visit.combat_preview { 4 } else { 0 })) {
                crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
                break;
            }
            if options.frames.is_some_and(|limit| rendered >= limit) {
                if let Some(path) = &options.capture { crate::viewer::save_capture(path)?; }
                break;
            }
            next_frame().await;
        }
        look.release();
        Ok(())
    }.await;
    if let Some(hosting) = hosting {
        hosting.cancel();
        let started = Instant::now();
        while !matches!(hosting.status(), Status::Stopped | Status::Failed(_)) && started.elapsed() < Duration::from_secs(5) {
            next_frame().await;
        }
        if let Some(path) = &launch.invite_file {
            let _ = std::fs::remove_file(path);
        }
    }
    let _ = std::fs::remove_file(crate::preferences::path("coop-live.json"));
    show_mouse(true);
    result
}
