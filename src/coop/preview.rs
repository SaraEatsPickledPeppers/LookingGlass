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
use crate::{assets::Assets, character::Character, inventory::Stats, movement::FIXED_DT, Options};
use anyhow::{Context, Result};
use macroquad::prelude::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct Launch {
    pub host: bool,
    pub internet: bool,
    pub invite_file: PathBuf,
    pub drive: bool,
}
pub async fn run(assets: &mut Assets, options: &Options, launch: &Launch) -> Result<()> {
    let ui = crate::ui::Ui::load(assets)?;
    let mut hosting = None;
    let result = async {
        let identity = Identity::current(assets.fingerprint()?);
        let (mut host, mut guest, invite) = if launch.host {
            hosting = Some(Hosting::start(crate::preferences::path("coop"),
                PathBuf::from("private/coop/helpers"), launch.internet, Conditions::default())?);
            let setup = hosting.as_ref().unwrap();
            let (peer, invite) = loop {
                if is_quit_requested() || is_key_pressed(KeyCode::Escape) { return Ok(()); }
                if let Some(ready) = setup.take_ready()? { break ready; }
                clear_background(BLACK);
                ui.toast(match setup.status() {
                    Status::PreparingHelper => "Preparing internet hosting...",
                    _ => "Creating your invite... Esc to cancel",
                }, screen_height() * 0.5);
                next_frame().await;
            };
            let visit = Visit { epoch: 1, map: options.map.clone(), entry: options.entry.clone(), difficulty: options.difficulty };
            let session = Host::new(peer, identity, visit)?;
            if let Some(parent) = launch.invite_file.parent() { std::fs::create_dir_all(parent)?; }
            std::fs::write(&launch.invite_file, invite.link())?;
            (Some(session), None, Some(invite))
        } else {
            let invite = Invite::parse(&std::fs::read_to_string(&launch.invite_file).context("The host invite is not ready.")?)?;
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
            ui.toast("Joining the host... Esc to cancel", screen_height() * 0.5);
            next_frame().await;
        }
        let visit = visit.unwrap();
        clear_background(BLACK);
        ui.toast("Opening Wonderland for two...", screen_height() * 0.5);
        next_frame().await;
        let mut scene = crate::render::Scene::load_for(assets, &visit.map, visit.difficulty)?;
        let mut characters = [Character::load_for_player(assets, PlayerId::Blue)?,
            Character::load_for_player(assets, PlayerId::Red)?];
        let (eye, initial_yaw) = crate::interaction::spawn(&scene.map, visit.entry.as_deref());
        let eye = options.start_at.map_or(eye, |p| p + Vec3::Z * crate::movement::EYE_HEIGHT);
        let mut stats = Stats::for_level(&visit.map, visit.entry.as_deref());
        stats.difficulty = visit.difficulty;
        let mut roster = if launch.host { Some(Roster::new(&scene.world, eye, initial_yaw, stats)?) } else { None };
        if let Some(host) = &mut host { host.loaded(visit.epoch)?; }
        if let Some(guest) = &mut guest { guest.loaded(visit.epoch)?; }
        let mut display = roster.as_ref().map(|r| r.frame(visit.epoch, 0));
        let mut presented: Option<Frame> = None;
        let mut yaw = initial_yaw;
        let mut pitch = 0.;
        let mut look = crate::look::MouseLook::default();
        let prefs = crate::preferences::Preferences::load();
        let mut controls = crate::input::Input::default();
        let mut camera = crate::camera::Follow::default();
        let mut local_latch = RemoteInput::default();
        let mut keys = Input::default();
        let mut local_sequence = 0;
        let mut accumulator = 0_f64;
        let mut menu = false;
        let mut last_sent = 0;
        let mut rendered = 0;
        let mut started_playing = None;
        let mut report_at = Instant::now();
        let mut guest_send_at = Instant::now();
        let mut reset_characters = true;
        let mut captured_midpoint = false;
        loop {
            let now = Instant::now();
            if let Some(setup) = &hosting {
                if let Status::Failed(reason) = setup.status() { anyhow::bail!("{reason}"); }
            }
            if let Some(host) = &mut host { host.poll(now)?; }
            if let Some(guest) = &mut guest {
                guest.poll(now)?;
                if let Some(frame) = guest.take_frame() { display = Some(frame); }
            }
            let playing = host.as_ref().map_or_else(|| guest.as_ref().unwrap().phase() == Phase::Playing,
                |h| h.phase() == Phase::Playing);
            if playing { started_playing.get_or_insert(now); }
            let elapsed = get_frame_time().clamp(0., 0.1);
            let focused = crate::look::window_focused();
            controls.update(&prefs, focused);
            if focused && controls.ui(KeyCode::Escape) { menu = !menu; }
            if is_quit_requested() || menu && focused && is_key_pressed(KeyCode::Q) { break; }
            if focused && is_key_pressed(KeyCode::F8) {
                if let Some(invite) = &invite { miniquad::window::clipboard_set(&invite.link()); }
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
            // Deterministic local test input still travels through the ordinary
            // session and host simulation, even while this window lacks focus.
            if launch.drive && playing {
                let age = started_playing.unwrap().elapsed().as_secs_f32();
                keys.run = false;
                keys.wish = if (2. ..4.).contains(&age) { if launch.host { Vec2::Y } else { Vec2::X } } else { Vec2::ZERO };
                keys.swim = keys.wish.extend(0.);
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
                        roster.step(&scene.world, [local_latch.sample(now), host.controls(now)])?;
                        accumulator -= f64::from(FIXED_DT);
                    }
                    let frame = roster.frame(visit.epoch, host.acknowledged_input());
                    if frame.tick >= last_sent + 6 && host.send_frame(frame.clone())? { last_sent = frame.tick; }
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
                for (index, (character, body)) in characters.iter_mut().zip(&visual.players).enumerate() {
                    if reset_characters || presented.as_ref().is_some_and(|previous|
                        previous.players[index].stats.alive() != body.stats.alive()
                        || previous.players[index].player.feet.distance(body.player.feet) > 128.) {
                        character.reset(&body.player, body.yaw);
                    }
                    let aim = vec3(body.yaw.cos() * body.pitch.cos(), body.yaw.sin() * body.pitch.cos(), body.pitch.sin());
                    character.update(elapsed, &body.player, body.running,
                        crate::weapons::WeaponInput { selected: crate::weapons::UNARMED, aim, ..Default::default() },
                        &crate::combat::Context { world: &scene.world, targets: &[] });
                    if !body.stats.alive() { character.die(); }
                    character.atmosphere(&scene.atmosphere, camera_eye);
                    if index != watched.index() || show_local {
                        character.draw(body.player.feet, false);
                    }
                }
                reset_characters = false;
                scene.draw(camera_eye, frame.tick as f32 * FIXED_DT, false, true, &[]);
                presented = Some(visual);
                set_default_camera();
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
                    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
                    std::fs::write(path, serde_json::to_vec(&serde_json::json!({ "role": if launch.host { "host" } else { "guest" },
                        "pid": std::process::id(), "playing": playing, "frame": frame, "focused": focused,
                        "camera_player": watched, "camera": camera_eye }))?)?;
                    report_at = now;
                }
            }
            set_default_camera();
            ui.toast(if playing { "Co-op movement test / campaign integration in progress" }
                else if started_playing.is_some() { "Your partner left. Share the invite to rejoin." }
                else { "Waiting for the other player to finish loading..." }, screen_height() - 110.);
            ui.toast(if menu { "Esc: resume / Q: leave co-op" } else if launch.host { "Esc: menu / F8: copy invite" } else { "Esc: menu" }, screen_height() - 50.);
            if focused && is_key_pressed(KeyCode::F12) {
                crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
            }
            rendered += u64::from(playing);
            if launch.drive && !captured_midpoint && started_playing.is_some_and(|start| start.elapsed() >= Duration::from_secs(6)) {
                crate::viewer::save_capture(&crate::preferences::path("coop-midpoint.png"))?;
                captured_midpoint = true;
            }
            if launch.drive && started_playing.is_some_and(|start| start.elapsed()
                >= Duration::from_secs(if launch.host { 12 } else { 10 })) {
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
        while hosting.status() != Status::Stopped && started.elapsed() < Duration::from_secs(5) {
            next_frame().await;
        }
        let _ = std::fs::remove_file(&launch.invite_file);
    }
    result
}
