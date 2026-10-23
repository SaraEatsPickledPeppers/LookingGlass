//! Host-owned bodies and individual resources. Shared controllers must advance
//! once around this per-player step, never once for each client/render window.
use super::{
    protocol::Input,
    rules::{DeathEvent, Lives, RESPAWN_HEALTH},
    PlayerId,
};
use crate::{
    collision::World,
    inventory::Stats,
    movement::{Controls, Player, EYE_HEIGHT, FIXED_DT},
};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Body {
    pub player: Player,
    pub stats: Stats,
    pub yaw: f32,
    pub pitch: f32,
    pub running: bool,
}
impl Body {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.player.feet.is_finite()
                && self.player.velocity.is_finite()
                && self.yaw.is_finite()
                && self.pitch.is_finite()
                && self.pitch.abs() <= 1.56,
            "Invalid co-op player state."
        );
        self.player.validate_save()?;
        self.stats.validate_save()
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub epoch: u64,
    pub tick: u64,
    pub acknowledged_input: u64,
    pub players: [Body; 2],
    pub lives: Lives,
}
impl Frame {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.epoch > 0, "Invalid co-op visit.");
        for body in &self.players {
            body.validate()?;
        }
        self.lives.validate()
    }
}

pub struct Roster {
    pub players: [Body; 2],
    pub lives: Lives,
    pub tick: u64,
    checkpoint: [Body; 2],
}
impl Roster {
    pub fn new(world: &World, eye: Vec3, yaw: f32, stats: Stats) -> Result<Self> {
        let blue = Player::spawn(world, eye).context("Alice's entrance is obstructed.")?;
        let red = safe_near(world, blue.feet)
            .context("There is no safe space for player two at this entrance.")?;
        let players = [blue, red].map(|player| Body {
            player,
            stats: stats.clone(),
            yaw,
            pitch: 0.,
            running: false,
        });
        Ok(Self {
            checkpoint: players.clone(),
            players,
            lives: Default::default(),
            tick: 0,
        })
    }
    pub fn step(&mut self, world: &World, inputs: [(Controls, Input); 2]) -> Result<()> {
        self.tick += 1;
        for (body, (controls, look)) in self.players.iter_mut().zip(inputs) {
            if !body.stats.alive() {
                continue;
            }
            body.yaw = look.yaw;
            body.pitch = look.pitch;
            body.running = controls.run;
            body.stats.update(FIXED_DT);
            body.stats.prepare_player(&mut body.player);
            let landings = body.player.landings;
            body.player.tick(world, controls);
            body.stats
                .damage(std::mem::take(&mut body.player.liquid_damage));
            if body.player.landings != landings {
                body.stats.fall(body.player.landing_speed);
            }
            if crate::recovery::outside_world(world, body.player.feet) {
                body.stats.damage(10000.);
            }
        }
        let events = self
            .lives
            .tick(self.players.each_ref().map(|p| p.stats.alive()), false);
        if events.contains(&DeathEvent::RestoreCheckpoint) {
            self.players = self.checkpoint.clone();
            ensure!(
                self.players.iter().all(|p| world.body_clear(p.player.feet)),
                "Co-op checkpoint is obstructed."
            );
            self.lives.checkpoint_restored([true; 2]);
        } else if let Some((dead, survivor)) = self.lives.pending_spawn() {
            if let Some(player) = safe_near(world, self.players[survivor.index()].player.feet) {
                let body = &mut self.players[dead.index()];
                body.player = player;
                body.stats.set_health(RESPAWN_HEALTH)?;
            }
        }
        Ok(())
    }
    pub fn frame(&self, epoch: u64, acknowledged_input: u64) -> Frame {
        Frame {
            epoch,
            tick: self.tick,
            acknowledged_input,
            players: self.players.clone(),
            lives: self.lives.clone(),
        }
    }
}

/// Grounded, clear placement close to the survivor, reached without crossing a
/// wall. Do not spawn on a ledge, in a liquid, on the other body or over a drop.
pub fn safe_near(world: &World, feet: Vec3) -> Option<Player> {
    for radius in [48., 72., 96.] {
        for direction in [
            Vec3::X,
            -Vec3::X,
            Vec3::Y,
            -Vec3::Y,
            vec3(1., 1., 0.).normalize(),
            vec3(-1., 1., 0.).normalize(),
            vec3(1., -1., 0.).normalize(),
            vec3(-1., -1., 0.).normalize(),
        ] {
            let candidate = feet + direction * radius + Vec3::Z * 18.;
            let path = world.body_trace(feet + Vec3::Z * 1., candidate);
            if path.start_solid || path.fraction < 1. {
                continue;
            }
            let floor = world.body_trace(candidate, candidate - Vec3::Z * 50.);
            if floor.start_solid || floor.fraction >= 1. || floor.normal.z < 0.65 {
                continue;
            }
            let landing = candidate - Vec3::Z * (50. * floor.fraction);
            if !crate::recovery::supported(world, landing) {
                continue;
            }
            return Player::spawn(world, landing + Vec3::Z * EYE_HEIGHT);
        }
    }
    None
}

pub fn camera_player(frame: &Frame, local: PlayerId) -> PlayerId {
    if frame.players[local.index()].stats.alive() {
        local
    } else {
        local.partner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn floor() -> World {
        World::fixture(&[(vec3(-1000., -1000., -100.), vec3(1000., 1000., 0.))])
    }
    #[test]
    fn two_bodies_share_the_host_clock_but_keep_separate_motion_and_health() {
        let world = floor();
        let mut roster =
            Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        let start = roster.players[1].player.feet;
        for _ in 0..120 {
            roster
                .step(
                    &world,
                    [
                        (
                            Controls {
                                wish: Vec2::Y,
                                ..Default::default()
                            },
                            Input::default(),
                        ),
                        Default::default(),
                    ],
                )
                .unwrap();
        }
        assert!((roster.players[0].player.feet.y - 104.).abs() < 5.);
        assert!(roster.players[1].player.feet.distance(start) < 0.1);
        roster.players[1].stats.damage(10.);
        assert_eq!(roster.players[0].stats.sanity(), 100.);
        assert_eq!(roster.players[1].stats.sanity(), 90.);
    }
    #[test]
    fn death_spectates_then_respawns_and_a_double_death_restores_both() {
        let world = floor();
        let mut roster =
            Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        roster.players[1].stats.damage(1000.);
        roster.step(&world, Default::default()).unwrap();
        assert_eq!(
            camera_player(&roster.frame(1, 0), PlayerId::Red),
            PlayerId::Blue
        );
        for _ in 0..1199 {
            roster.step(&world, Default::default()).unwrap();
        }
        assert!(!roster.players[1].stats.alive());
        roster.step(&world, Default::default()).unwrap();
        assert_eq!(roster.players[1].stats.sanity(), RESPAWN_HEALTH);
        assert!(
            roster.players[0]
                .player
                .feet
                .distance(roster.players[1].player.feet)
                >= 40.
        );
        for body in &mut roster.players {
            body.stats.damage(1000.);
        }
        roster.step(&world, Default::default()).unwrap();
        assert!(roster.players.iter().all(|p| p.stats.sanity() == 100.));
    }
    #[test]
    fn no_safe_neighbor_does_not_place_a_body_over_a_drop() {
        let small = World::fixture(&[(vec3(-20., -20., -100.), vec3(20., 20., 0.))]);
        assert!(safe_near(&small, Vec3::Z * 0.04).is_none());
    }
}
