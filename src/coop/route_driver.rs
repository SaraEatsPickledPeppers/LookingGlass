//! Explicit local test controller. Only ordinary client inputs leave this module;
//! it cannot set positions, health, inventory, encounters or host world state.
//! Each process plans from its own received view. The harness supplies goals via
//! that process's isolated profile, never authoritative state through the wire.
use super::{arena::Arena, protocol::Input, world::Frame, PlayerId};
use crate::{collision::World, movement::{Controls, Player}, route::Route};
use anyhow::{ensure, Result};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, path::PathBuf, sync::mpsc, time::{Duration, Instant, SystemTime}};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    id: u64,
    heartbeat: u64,
    epoch: u64,
    goal: Option<Vec3>,
    #[serde(default)]
    marker: Option<usize>,
    #[serde(default)]
    horizontal: bool,
    #[serde(default)]
    rope: Option<RopeCommand>,
    #[serde(default)]
    run: bool,
    #[serde(default)]
    jump: bool,
    navigate: bool,
    attack: bool,
    capture: bool,
    finish: bool,
}
impl Command {
    fn validate(&self) -> Result<()> {
        ensure!(self.id > 0 && self.epoch > 0 && self.heartbeat > 0, "Invalid route command identity");
        ensure!(self.goal.is_none_or(|p| p.is_finite() && p.abs().max_element() < 1_000_000.),
            "Invalid route goal");
        ensure!(self.goal.is_none() || self.marker.is_none(), "Choose a route goal or a marker");
        ensure!(self.marker.is_none_or(|i| i < 100_000), "Invalid route marker");
        ensure!(!(self.run || self.jump) || !self.navigate && self.rope.is_none(),
            "Explicit run/jump needs straight movement, not a planned route or rope command");
        if let Some(rope) = &self.rope { rope.validate()?; }
        Ok(())
    }
}
#[derive(Clone, Deserialize)]
#[serde(tag="action", rename_all="snake_case", deny_unknown_fields)]
enum RopeCommand {
    Approach { goal: Vec2 },
    Climb { rise: f32 },
    Release { goal: Vec2 },
}
impl RopeCommand {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Approach { goal } | Self::Release { goal } => ensure!(goal.is_finite() && goal.abs().max_element() < 1_000_000., "Invalid rope goal"),
            Self::Climb { rise } => ensure!(rise.is_finite() && rise.abs() <= 1., "Invalid rope climb input"),
        }
        Ok(())
    }
    fn sample(&self, player: &Player, world: &World, keys: &mut Input, jumped: &mut bool, used: &mut bool) -> (&'static str, bool) {
        match self {
            Self::Approach { goal } => {
                if player.rope.is_some() { return ("holding rope", true); }
                keys.wish = (*goal - player.feet.truncate()).normalize_or_zero(); keys.run = true;
                if !*jumped { keys.jumps += 1; *jumped = true; }
                // One edge per command: an unacknowledged grab must never turn
                // into a second E press that releases a delayed guest's grip.
                if !*used && player.rope_prompt(world).is_some() { keys.uses += 1; *used = true; }
                ("approaching rope", false)
            }
            Self::Climb { rise } => {
                keys.rise = *rise;
                ("climbing rope", player.rope.is_some())
            }
            Self::Release { goal } => {
                if player.rope.is_some() && !*used { keys.uses += 1; *used = true; }
                if player.rope.is_none() && player.grounded { return ("landed from rope", true); }
                // Steer away during the drop, otherwise the automatic nearby
                // grab can catch the same rope again before a route can plan.
                keys.wish = (*goal - player.feet.truncate()).normalize_or_zero(); keys.run = true;
                ("releasing rope", false)
            }
        }
    }
}
#[derive(Default, Serialize)]
pub struct Report {
    pub command: Option<u64>,
    pub phase: &'static str,
    pub reached: bool,
    pub captured: Option<u64>,
    pub replans: usize,
    pub error: Option<String>,
    pub observed_epoch: u64,
    pub observed_tick: u64,
    pub scene: Option<String>,
    pub speech: bool,
    pub clear: [bool; 2],
    pub blade_pickups: [bool; 2],
    pub cards_pickups: [bool; 2],
    pub expected: Option<Vec3>,
    pub plan_remaining: usize,
    pub jumps: u64,
    /// Read-only encounter diagnostics. Selection exists only on the host;
    /// guests restore enemy presentation without running another AI simulation.
    pub guards: Vec<GuardReport>,
    pub network_states: VecDeque<NetworkState>,
}
#[derive(Serialize)]
pub struct NetworkState {
    epoch: u64,
    tick: u64,
    sha256: String,
}
#[derive(Serialize)]
pub struct GuardReport {
    name: String,
    active: bool,
    feet: Vec3,
    health: f32,
    state: crate::combat::State,
    selected_player: Option<usize>,
    distance: [f32; 2],
    line_of_sight: [bool; 2],
}
struct Worker {
    id: u64,
    epoch: u64,
    warped: u64,
    start: Vec3,
    result: mpsc::Receiver<Result<Vec<(Controls, Vec3)>, usize>>,
}
pub struct Driver {
    path: PathBuf,
    command: Option<Command>,
    modified: Option<SystemTime>,
    polled: Instant,
    refreshed: Instant,
    used: Instant,
    worker: Option<Worker>,
    plan: VecDeque<(Controls, Vec3)>,
    step_tick: u64,
    step_at: Instant,
    jumped: bool,
    rope_used: bool,
    location: Option<(u64, u64, u64)>,
    pub report: Report,
}
impl Driver {
    /// Compare actual sent/restored states, independent of the two report clocks.
    /// Only the opt-in local harness keeps this bounded diagnostic history.
    pub fn record_state(&mut self, frame: &Frame, arena: &Arena) -> Result<()> {
        use sha2::{Digest, Sha256};
        let states = &mut self.report.network_states;
        if states.back().is_some_and(|s| (s.epoch,s.tick) == (frame.epoch,frame.tick)) { return Ok(()); }
        let enemies = arena.stage.interactions.encounters.as_ref().map(|c| c.snapshot());
        let bytes = serde_json::to_vec(&(frame.epoch, frame.tick, frame.checkpoint_generation,
            &frame.players, &frame.lives, enemies))?;
        states.push_back(NetworkState { epoch:frame.epoch, tick:frame.tick,
            sha256:format!("{:x}",Sha256::digest(bytes)) });
        while states.len() > 64 { states.pop_front(); }
        Ok(())
    }
    pub fn new(path: PathBuf) -> Self {
        let now = Instant::now();
        Self { path, command: None, modified: None, polled: now, refreshed: now, used: now,
            worker: None, plan: VecDeque::new(), step_tick: 0, step_at: now, jumped: false, rope_used: false,
            location: None, report: Report::default() }
    }
    pub fn poll(&mut self, now: Instant) {
        if now.duration_since(self.polled) < Duration::from_millis(50) { return; }
        self.polled = now;
        let read = || -> Result<Option<(SystemTime, Command)>> {
            let meta = match self.path.metadata() { Ok(meta) => meta, Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None), Err(e) => return Err(e.into()) };
            ensure!(meta.len() <= 4096, "Route command is too large");
            let modified = meta.modified()?;
            if Some(modified) == self.modified { return Ok(None); }
            let command: Command = serde_json::from_slice(&std::fs::read(&self.path)?)?;
            command.validate()?;
            Ok(Some((modified, command)))
        };
        match read() {
            Ok(Some((modified, command))) => {
                if self.command.as_ref().is_none_or(|old| old.id != command.id) {
                    self.plan.clear(); self.jumped = false; self.rope_used = false; self.location = None;
                    self.report.error = None;
                }
                self.report.command = Some(command.id);
                self.command = Some(command); self.modified = Some(modified); self.refreshed = now;
            }
            Ok(None) => (),
            Err(error) => { self.report.error = Some(error.to_string()); self.command = None; self.plan.clear(); }
        }
    }
    fn fresh(&self, now: Instant) -> bool { now.duration_since(self.refreshed) < Duration::from_secs(3) }
    fn neutral(keys: &mut Input) {
        keys.wish = Vec2::ZERO; keys.swim = Vec3::ZERO; keys.rise = 0.;
        keys.run = false; keys.primary = false; keys.alternate = false; keys.weapon = None;
    }
    // Observe after simulation, alongside the frame being written. Input sampling
    // happens before a host step and can otherwise label a finished scene with
    // its previous name but the new (empty) owner.
    pub fn observe(&mut self, frame: &Frame, arena: &Arena, world: &World) {
        self.report.observed_epoch = frame.epoch;
        self.report.observed_tick = frame.tick;
        self.report.clear = frame.players.each_ref().map(|b| !b.stats.alive() || world.body_clear(b.player.feet));
        self.report.blade_pickups = frame.players.each_ref().map(|b| b.stats.copies(0) > 0
            && arena.pickups.iter().any(|p| matches!(p.kind, crate::inventory::PickupKind::Weapon(0))
                && b.stats.collected.contains(&p.id)));
        self.report.cards_pickups = frame.players.each_ref().map(|b| b.stats.copies(1) > 0
            && arena.pickups.iter().any(|p| matches!(p.kind, crate::inventory::PickupKind::Weapon(1))
                && b.stats.collected.contains(&p.id)));
        self.report.scene = arena.stage.interactions.village.as_ref().and_then(|v|v.cinema.scene_id())
            .or_else(|| arena.stage.interactions.pandemonium.as_ref().and_then(|p|p.scene_id()))
            .or_else(|| arena.stage.interactions.fortress.as_ref().and_then(|f|f.state.cinema.scene_id())).map(str::to_owned);
        self.report.speech = arena.stage.story.busy() && !arena.stage.busy();
        self.report.guards = arena.stage.interactions.encounters.iter().flat_map(|cast| &cast.actors)
            .filter_map(|actor| {
                let crate::encounters::Enemy::Guard(guard) = &actor.enemy else { return None; };
                let center = guard.target(0).center;
                Some(GuardReport {
                    name: actor.name.clone(), active: actor.active && actor.enabled,
                    feet: guard.feet, health: guard.health, state: guard.state,
                    selected_player: guard.opponents.selected_player,
                    distance: frame.players.each_ref().map(|b| center.distance(b.player.eye())),
                    line_of_sight: frame.players.each_ref().map(|b| {
                        let trace = world.sweep(center, b.player.eye(), Vec3::splat(0.5));
                        b.stats.alive() && !b.stats.ignores_alice() && !trace.start_solid && trace.fraction >= 1.
                    }),
                })
            }).collect();
    }
    pub fn sample(&mut self, frame: &Frame, arena: &Arena, world: &World, map: &crate::bsp::Bsp, local: PlayerId, keys: &mut Input, now: Instant) {
        Self::neutral(keys);
        self.report.reached = false;
        self.report.expected = self.plan.front().map(|(_, at)| *at);
        self.report.plan_remaining = self.plan.len();
        self.report.jumps = keys.jumps;
        self.report.phase = "idle";
        let body = &frame.players[local.index()];
        let location = (frame.epoch, body.warped_at, body.spawned_at);
        if self.location != Some(location) { self.plan.clear(); self.jumped = false; self.location = Some(location); }
        // Never spawn a second worker while a superseded plan is still running.
        if let Some(worker) = &self.worker {
            let result = match worker.result.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(0)),
                Err(mpsc::TryRecvError::Empty) => None,
            };
            if let Some(result) = result {
                if self.command.as_ref().is_some_and(|c| c.id == worker.id && c.epoch == worker.epoch)
                    && frame.epoch == worker.epoch && body.warped_at == worker.warped
                    && body.player.feet.distance(worker.start) < 16. && !arena.stage.busy() {
                    match result {
                        Ok(plan) => {
                            let trace = serde_json::json!({ "build": env!("LOOKING_GLASS_BUILD"),
                                "epoch": worker.epoch, "command": worker.id, "start": worker.start,
                                "steps": plan.iter().map(|(input, expected)|serde_json::json!({
                                    "wish": input.wish, "jump": input.jump, "run": input.run, "expected": expected
                                })).collect::<Vec<_>>() });
                            if let Ok(bytes) = serde_json::to_vec(&trace) {
                                let _ = std::fs::write(crate::preferences::path(&format!("route-plan-{}-{}.json",worker.id,self.report.replans)),bytes);
                            }
                            self.plan = plan.into(); self.step_tick = frame.tick; self.step_at = now; self.jumped = false;
                        }
                        Err(0) => self.report.error = Some("Route planning worker ended without a result".into()),
                        Err(states) => self.report.error = Some(format!("No route after {states} physics states from {:?}", worker.start)),
                    }
                }
                self.worker = None;
            }
        }
        let Some(command) = self.command.as_ref().filter(|c| c.epoch == frame.epoch && self.fresh(now)) else {
            self.report.phase = "waiting for command"; return;
        };
        if !body.stats.alive() { self.report.phase = "spectating"; return; }
        if arena.stage.story.busy() && arena.stage.owner == Some(local)
            && now.duration_since(self.used) >= Duration::from_millis(250) {
            keys.uses += 1; self.used = now;
        }
        if arena.stage.busy() {
            self.plan.clear(); self.jumped = false; self.report.phase = "scene";
            return;
        }
        if let Some(rope) = &command.rope {
            let (phase, reached) = rope.sample(&body.player, world, keys, &mut self.jumped, &mut self.rope_used);
            self.report.phase = phase; self.report.reached = reached;
            if keys.wish.length() > 0. { keys.yaw = keys.wish.y.atan2(keys.wish.x); keys.pitch = 0.; }
            return;
        }
        if command.attack && body.stats.copies(0) > 0 {
            let mut targets = arena.npcs.targets();
            if let Some(cast) = &arena.stage.interactions.encounters { targets.extend(cast.targets()); }
            targets.extend(arena.stage.interactions.shot_targets());
            let context = crate::combat::Context { world, targets: &targets };
            if let Some(target) = targets.iter().filter(|t|t.center.distance(body.player.eye()) < 1000.)
                .filter(|t|crate::combat::contact(&context,body.player.eye(),t.center,1.).is_some_and(|(id,_)|id==t.id))
                .min_by(|a,b|a.center.distance_squared(body.player.eye()).total_cmp(&b.center.distance_squared(body.player.eye()))) {
                let delta = target.center-body.player.eye();
                keys.yaw = delta.y.atan2(delta.x); keys.pitch = (delta.z/delta.length().max(1.)).asin().clamp(-1.56,1.56);
                keys.weapon = Some(0); keys.primary = delta.length()<80.; keys.alternate = !keys.primary;
            }
        }
        let goal = if let Some(marker) = command.marker {
            match map.entities.get(marker).and_then(|e| e.get("origin")).and_then(|s| crate::interaction::vector(s))
                .filter(|p| p.is_finite() && p.abs().max_element() < 1_000_000.) {
                Some(goal) => Some(goal),
                None => { self.report.error = Some(format!("Missing route marker {marker}")); return; }
            }
        } else { command.goal };
        let Some(goal) = goal else { return; };
        let delta = goal-body.player.feet;
        self.report.reached = if command.navigate { delta.length()<42. && body.player.grounded }
            else { delta.truncate().length()<8. && (command.horizontal || delta.z.abs()<35.) };
        if self.report.reached { self.report.phase="reached"; self.plan.clear(); return; }
        if command.navigate {
            self.report.phase="navigating";
            // A host observes its own simulation clock directly. A guest sends
            // 200 ms input segments on its local clock; timing them from received
            // snapshots would turn delivery jitter into uneven key-hold lengths.
            while !self.plan.is_empty() && if local == PlayerId::Blue {
                frame.tick.saturating_sub(self.step_tick) >= 24
            } else { now.duration_since(self.step_at) >= Duration::from_millis(200) } {
                let (_, expected) = self.plan.pop_front().unwrap();
                self.step_tick += 24; self.step_at += Duration::from_millis(200); self.jumped=false;
                // The host can check a landing immediately. Guest snapshots
                // still show the position before its in-flight inputs: comparing
                // them here restarts a valid plan and duplicates its first move.
                // A guest waits for the full sequence and a settled body before
                // planning again from the resulting authoritative position.
                if local == PlayerId::Blue && body.player.grounded && body.player.feet.distance(expected) > 24. {
                    self.plan.clear();
                }
            }
            if let Some((input,_)) = self.plan.front() {
                keys.wish=input.wish; keys.swim=input.swim; keys.rise=input.rise; keys.run=input.run;
                if input.jump && !self.jumped {
                    keys.jumps+=1; self.jumped=true;
                }
            } else if self.worker.is_none() && self.report.error.is_none()
                && (body.player.grounded || body.player.ledge.is_some()) && body.player.velocity.truncate().length()<20. {
                let (send,result)=mpsc::channel();
                let world=world.clone(); let player:Player=body.player.clone();
                self.worker=Some(Worker { id:command.id, epoch:frame.epoch, warped:body.warped_at, start:player.feet,result });
                self.report.replans+=1;
                std::thread::spawn(move || { let plan=Route::plan(&world,&player,goal,22.)
                    .or_else(|_|Route::plan(&world,&player,goal,40.)); let _=send.send(plan); });
            }
            if self.worker.is_some() { self.report.phase="planning"; }
        } else {
            self.report.phase="walking";
            keys.wish=delta.truncate().normalize_or_zero()*(delta.truncate().length()/48.).min(1.);
            keys.swim=keys.wish.extend(0.);
            keys.run=command.run;
            if command.jump && !self.jumped { keys.jumps+=1; self.jumped=true; }
        }
        if !keys.primary && !keys.alternate && keys.wish.length()>0. {
            keys.yaw=keys.wish.y.atan2(keys.wish.x); keys.pitch=0.;
        }
        if arena.stage.interactions.prompt(world, body.player.eye(), super::arena::direction(keys.yaw,keys.pitch)) == Some("E  open door")
            && now.duration_since(self.used) >= Duration::from_millis(300) {
            keys.uses += 1; self.used = now;
        }
    }
    pub fn capture_and_finish(&mut self, now: Instant) -> Result<bool> {
        let Some(command)=self.command.as_ref().filter(|_|self.fresh(now)) else { return Ok(false); };
        if (command.capture || command.finish) && self.report.captured != Some(command.id) {
            crate::viewer::save_capture(&crate::preferences::path(&format!("route-capture-{}.png",command.id)))?;
            self.report.captured=Some(command.id);
        }
        Ok(command.finish)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delayed_rope_acknowledgement_does_not_repeat_grab_or_release_edges() {
        let mut world = World::fixture(&[]);
        world.traversal.ropes.push(crate::traversal::Rope { id:crate::entity::Id(1), model:1, enabled:true,
            origin:Vec3::ZERO, anchor:Vec3::Z*320., length:320. });
        let mut player = Player::new(vec3(24.,0.,0.));
        let mut keys = Input::default();
        let (mut jumped, mut used) = (false,false);
        let approach = RopeCommand::Approach { goal:Vec2::ZERO };
        for _ in 0..30 {
            Driver::neutral(&mut keys);
            assert!(!approach.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        }
        assert_eq!((keys.jumps,keys.uses),(1,1));
        player.rope = Some(crate::movement::RopeGrip { id:crate::entity::Id(1),length:280.,rise:0. });
        Driver::neutral(&mut keys);
        assert!(approach.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        assert_eq!(keys.wish,Vec2::ZERO);
        let release = RopeCommand::Release { goal:Vec2::X*300. }; used=false;
        for _ in 0..30 { Driver::neutral(&mut keys); release.sample(&player,&world,&mut keys,&mut jumped,&mut used); }
        assert_eq!(keys.uses,2);
        player.rope=None; player.grounded=false;
        Driver::neutral(&mut keys);
        assert!(!release.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        assert!(keys.run && keys.wish.x>0.);
        player.grounded=true; Driver::neutral(&mut keys);
        assert!(release.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        assert_eq!(keys.wish,Vec2::ZERO);
    }
    #[test]
    fn legacy_route_commands_remain_valid_and_rope_inputs_are_bounded() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,"navigate":false,"attack":true,"capture":false,"finish":false});
        let old:Command=serde_json::from_value(base.clone()).unwrap(); assert!(old.validate().is_ok());
        assert!(old.rope.is_none() && old.marker.is_none() && !old.horizontal && !old.run && !old.jump);
        let mut jumping=base.clone(); jumping["jump"]=true.into(); jumping["run"]=true.into();
        assert!(serde_json::from_value::<Command>(jumping.clone()).unwrap().validate().is_ok());
        jumping["navigate"]=true.into();
        assert!(serde_json::from_value::<Command>(jumping).unwrap().validate().is_err());
        for invalid in [serde_json::json!({"action":"climb","rise":2}),serde_json::json!({"action":"release","goal":[1e9,0]})] {
            let mut input=base.clone(); input["rope"]=invalid;
            assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
        }
        let mut input=base; input["goal"]=serde_json::json!([1,2,3]); input["marker"]=serde_json::json!(598);
        assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
    }
}
