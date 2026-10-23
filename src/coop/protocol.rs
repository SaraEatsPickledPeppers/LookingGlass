//! Gameplay messages are directional: guests can only identify themselves,
//! acknowledge a loaded visit, and supply bounded controls. Never accept a
//! guest's position, inventory, damage or world state as authoritative.
use crate::{movement::Controls, powerups::Difficulty};
use anyhow::{ensure, Result};
use macroquad::prelude::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub const REVISION: u32 = 1;
pub const INPUT_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_EDGES: u64 = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub protocol: u32,
    pub build: String,
    pub game_data: String,
}
impl Identity {
    pub fn current(game_data: String) -> Self {
        Self {
            protocol: REVISION,
            build: env!("LOOKING_GLASS_BUILD").into(),
            game_data,
        }
    }

    pub fn compatible(&self, other: &Self) -> Result<()> {
        ensure!(
            self.protocol == other.protocol && self.build == other.build,
            "Both players need the same Looking Glass co-op build."
        );
        ensure!(
            self.game_data == other.game_data,
            "Both players need matching game data and mods."
        );
        ensure!(
            valid_digest(&self.build) && valid_digest(&self.game_data),
            "The co-op build or game data identity is invalid."
        );
        Ok(())
    }
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// A host-issued epoch changes on every visit/reload, even re-entering the same
/// map. Old input and readiness acknowledgements cannot cross that boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Visit {
    pub epoch: u64,
    pub map: String,
    pub entry: Option<String>,
    pub difficulty: Difficulty,
    #[serde(default)]
    pub combat_preview: bool,
}
impl Visit {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.epoch > 0
                && self.map.len() <= 64
                && crate::interaction::destination(&self.map).is_some(),
            "The host selected an unknown campaign level."
        );
        ensure!(
            self.entry.as_ref().is_none_or(|entry| !entry.is_empty()
                && entry.len() <= 128
                && entry
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))),
            "The host selected an invalid level entrance."
        );
        ensure!(
            !self.combat_preview || self.map == "skool1" && self.entry.is_none(),
            "The combat fixture requires the school entrance."
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub wish: Vec2,
    pub swim: Vec3,
    pub rise: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub run: bool,
    pub primary: bool,
    pub alternate: bool,
    pub attack_presses: [u64; 2],
    pub weapon: Option<u8>,
    /// Cumulative edges survive a short press between network sends and queue
    /// backpressure. The host consumes each new edge once on its own clock.
    pub jumps: u64,
    pub uses: u64,
}
impl Input {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.wish.is_finite()
                && self.wish.length_squared() <= 1.001
                && self.swim.is_finite()
                && self.swim.length_squared() <= 1.001
                && self.rise.is_finite()
                && self.rise.abs() <= 1.
                && self.yaw.is_finite()
                && self.yaw.abs() <= std::f32::consts::PI
                && self.pitch.is_finite()
                && self.pitch.abs() <= 1.56
                && self.weapon.is_none_or(|w| w < 10),
            "Invalid co-op controls."
        );
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum FromGuest {
    Hello {
        identity: Identity,
    },
    Ready {
        epoch: u64,
    },
    Input {
        epoch: u64,
        sequence: u64,
        controls: Input,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum FromHost {
    Hello { identity: Identity },
    Enter { visit: Visit },
    Playing { epoch: u64 },
    Frame { frame: Box<super::world::Frame> },
}

/// Host-owned latch. Network traffic never advances the simulation clock.
#[derive(Default)]
pub struct RemoteInput {
    sequence: u64,
    last: Input,
    received: Option<Instant>,
    pending_jumps: u64,
    pending_uses: u64,
    pending_attacks: [u64; 2],
}
impl RemoteInput {
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn accept(&mut self, sequence: u64, input: Input, now: Instant) -> Result<()> {
        // Duplicates do not refresh the watchdog or replay button edges.
        if sequence <= self.sequence {
            return Ok(());
        }
        input.validate()?;
        let jumps = input.jumps.checked_sub(self.last.jumps);
        let uses = input.uses.checked_sub(self.last.uses);
        let attacks: [Option<u64>; 2] = std::array::from_fn(|i| {
            input.attack_presses[i].checked_sub(self.last.attack_presses[i])
        });
        ensure!(
            jumps.is_some_and(|n| n <= MAX_EDGES)
                && uses.is_some_and(|n| n <= MAX_EDGES)
                && attacks.iter().all(|n| n.is_some_and(|n| n <= MAX_EDGES)),
            "Invalid co-op button sequence."
        );
        ensure!(
            self.pending_jumps + jumps.unwrap() <= MAX_EDGES
                && self.pending_uses + uses.unwrap() <= MAX_EDGES
                && (0..2).all(|i| self.pending_attacks[i] + attacks[i].unwrap() <= MAX_EDGES),
            "Too many pending co-op actions."
        );
        self.pending_jumps += jumps.unwrap();
        self.pending_uses += uses.unwrap();
        for i in 0..2 {
            self.pending_attacks[i] += attacks[i].unwrap();
        }
        self.last = input;
        self.sequence = sequence;
        self.received = Some(now);
        Ok(())
    }

    pub fn sample(&mut self, now: Instant) -> (Controls, Input) {
        if self
            .received
            .is_none_or(|at| now.saturating_duration_since(at) >= INPUT_TIMEOUT)
        {
            self.pending_jumps = 0;
            self.pending_uses = 0;
            self.pending_attacks = [0; 2];
            // Preserve the last view, but never keep walking or attacking.
            return (
                Controls::default(),
                Input {
                    yaw: self.last.yaw,
                    pitch: self.last.pitch,
                    ..Input::default()
                },
            );
        }
        let jump = self.pending_jumps > 0;
        let use_pressed = self.pending_uses > 0;
        self.pending_jumps = self.pending_jumps.saturating_sub(1);
        self.pending_uses = self.pending_uses.saturating_sub(1);
        let mut input = self.last;
        input.primary |= self.pending_attacks[0] > 0;
        input.alternate |= self.pending_attacks[1] > 0;
        for n in &mut self.pending_attacks {
            *n = n.saturating_sub(1);
        }
        (
            Controls {
                wish: self.last.wish,
                swim: self.last.swim,
                rise: self.last.rise,
                run: self.last.run,
                jump,
                use_pressed,
            },
            input,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn attacks_survive_a_short_click_but_do_not_replay_or_outlive_input_timeout() {
        let now = Instant::now();
        let mut latch = RemoteInput::default();
        let input = Input {
            attack_presses: [1, 1],
            ..Default::default()
        };
        latch.accept(1, input, now).unwrap();
        let (_, sampled) = latch.sample(now);
        assert!(sampled.primary && sampled.alternate);
        latch.accept(1, input, now).unwrap();
        let (_, sampled) = latch.sample(now);
        assert!(!sampled.primary && !sampled.alternate);
        latch
            .accept(
                2,
                Input {
                    primary: true,
                    attack_presses: [2, 1],
                    ..input
                },
                now,
            )
            .unwrap();
        assert!(latch.sample(now).1.primary);
        assert!(latch.sample(now).1.primary);
        let (_, sampled) = latch.sample(now + INPUT_TIMEOUT);
        assert!(!sampled.primary && !sampled.alternate);
        assert!(
            latch.accept(3, input, now).is_err(),
            "Counters cannot roll backwards"
        );
    }
    #[test]
    fn controls_cannot_grant_speed_or_upload_world_state() {
        assert!(Input {
            wish: Vec2::splat(1.),
            ..Input::default()
        }
        .validate()
        .is_err());
        assert!(Input {
            yaw: f32::NAN,
            ..Input::default()
        }
        .validate()
        .is_err());
        assert!(Input {
            weapon: Some(10),
            ..Input::default()
        }
        .validate()
        .is_err());
        let mut forged = serde_json::to_value(FromGuest::Input {
            epoch: 1,
            sequence: 1,
            controls: Input::default(),
        })
        .unwrap();
        forged["position"] = serde_json::json!([0, 0, 0]);
        assert!(serde_json::from_value::<FromGuest>(forged).is_err());
        assert!(serde_json::from_str::<FromGuest>(r#"{"type":"Enter","visit":{}}"#).is_err());
    }
    #[test]
    fn short_presses_survive_coalescing_but_duplicates_never_repeat_them() {
        let now = Instant::now();
        let mut input = RemoteInput::default();
        input
            .accept(
                1,
                Input {
                    jumps: 2,
                    uses: 1,
                    ..Input::default()
                },
                now,
            )
            .unwrap();
        let (first, _) = input.sample(now);
        assert!(first.jump && first.use_pressed);
        input
            .accept(
                1,
                Input {
                    jumps: 2,
                    uses: 1,
                    ..Input::default()
                },
                now,
            )
            .unwrap();
        let (second, _) = input.sample(now);
        assert!(second.jump && !second.use_pressed);
        assert!(!input.sample(now).0.jump);
        assert!(input.accept(2, Input::default(), now).is_err());
    }
    #[test]
    fn stalled_or_replayed_input_releases_movement_and_attack() {
        let now = Instant::now();
        let mut input = RemoteInput::default();
        input
            .accept(
                1,
                Input {
                    wish: Vec2::X,
                    primary: true,
                    jumps: 1,
                    yaw: 1.,
                    ..Input::default()
                },
                now,
            )
            .unwrap();
        assert_eq!(input.sample(now).0.wish, Vec2::X);
        input
            .accept(1, Input::default(), now + INPUT_TIMEOUT)
            .unwrap();
        let (movement, buttons) = input.sample(now + INPUT_TIMEOUT);
        assert_eq!(movement.wish, Vec2::ZERO);
        assert!(!movement.jump && !buttons.primary);
        assert_eq!(buttons.yaw, 1.);
        input
            .accept(
                2,
                Input {
                    jumps: 1,
                    ..Input::default()
                },
                now + INPUT_TIMEOUT,
            )
            .unwrap();
        assert!(!input.sample(now + INPUT_TIMEOUT).0.jump);
    }
    #[test]
    fn different_builds_data_and_unknown_visits_are_rejected() {
        let current = Identity::current("0".repeat(64));
        current.compatible(&current).unwrap();
        let mut other = current.clone();
        other.build = "1".repeat(64);
        assert!(current.compatible(&other).is_err());
        other = current.clone();
        other.game_data = "2".repeat(64);
        assert!(current.compatible(&other).is_err());
        let mut visit = Visit {
            epoch: 1,
            map: "gvillage".into(),
            entry: None,
            difficulty: Difficulty::Normal,
            combat_preview: false,
        };
        visit.validate().unwrap();
        visit.map = "../../outside".into();
        assert!(visit.validate().is_err());
    }
}
