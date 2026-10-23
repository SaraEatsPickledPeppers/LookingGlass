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
        Ok(())
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
    pub scene: Option<String>,
    pub speech: bool,
    pub clear: [bool; 2],
    pub blade_pickups: [bool; 2],
    pub expected: Option<Vec3>,
    pub plan_remaining: usize,
    pub jumps: u64,
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
    jumped: bool,
    location: Option<(u64, u64, u64)>,
    pub report: Report,
}
impl Driver {
    pub fn new(path: PathBuf) -> Self {
        let now = Instant::now();
        Self { path, command: None, modified: None, polled: now, refreshed: now, used: now,
            worker: None, plan: VecDeque::new(), step_tick: 0, jumped: false,
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
                    self.plan.clear(); self.jumped = false; self.location = None;
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
    pub fn sample(&mut self, frame: &Frame, arena: &Arena, world: &World, local: PlayerId, keys: &mut Input, now: Instant) {
        Self::neutral(keys);
        self.report.reached = false;
        self.report.expected = self.plan.front().map(|(_, at)| *at);
        self.report.plan_remaining = self.plan.len();
        self.report.jumps = keys.jumps;
        self.report.clear = frame.players.each_ref().map(|b| !b.stats.alive() || world.body_clear(b.player.feet));
        self.report.blade_pickups = frame.players.each_ref().map(|b| b.stats.copies(0) > 0
            && arena.pickups.iter().any(|p| matches!(p.kind, crate::inventory::PickupKind::Weapon(0))
                && b.stats.collected.contains(&p.id)));
        self.report.scene = arena.stage.interactions.village.as_ref().and_then(|v|v.cinema.scene_id()).map(str::to_owned);
        self.report.speech = arena.stage.story.busy() && !arena.stage.busy();
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
                            self.plan = plan.into(); self.step_tick = frame.tick; self.jumped = false;
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
        let Some(goal) = command.goal else { return; };
        let delta = goal-body.player.feet;
        self.report.reached = if command.navigate { delta.length()<42. && body.player.grounded }
            else { delta.truncate().length()<8. && delta.z.abs()<35. };
        if self.report.reached { self.report.phase="reached"; self.plan.clear(); return; }
        if command.navigate {
            self.report.phase="navigating";
            // Measure segments on the observed simulation clock: asset/render
            // stalls must not consume an entire sequence without simulating it.
            while !self.plan.is_empty() && frame.tick.saturating_sub(self.step_tick) >= 24 {
                let (_, expected) = self.plan.pop_front().unwrap();
                self.step_tick += 24; self.jumped=false;
                // Allow for the sampled network view; abandon a missed landing
                // instead of blindly taking the next jump from the wrong place.
                if body.player.grounded && body.player.feet.distance(expected) > 24. {
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
        }
        if !keys.primary && !keys.alternate && keys.wish.length()>0. {
            keys.yaw=keys.wish.y.atan2(keys.wish.x); keys.pitch=0.;
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
