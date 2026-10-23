//! Nonblocking session lifecycle shared by the menu, simulation and probes.
//! A connected socket alone never means the guest is allowed to drive Alice.
use super::{
    protocol::{FromGuest, FromHost, Identity, Input, RemoteInput, Visit},
    transport::{Peer, State, MAX_INPUT, MAX_STATE},
};
use anyhow::{bail, ensure, Result};
use serde::Serialize;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const CHECK_TIMEOUT: Duration = Duration::from_secs(15);
const LOAD_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Waiting,
    Connecting,
    Checking,
    Loading,
    Playing,
    Ended,
}

fn queue<T: Serialize>(pending: &mut VecDeque<Vec<u8>>, message: &T) -> Result<()> {
    ensure!(pending.len() < 4, "Too many pending session changes.");
    let bytes = serde_json::to_vec(message)?;
    ensure!(
        bytes.len() <= MAX_INPUT,
        "Co-op session message is too large."
    );
    pending.push_back(bytes);
    Ok(())
}
fn flush(peer: &Peer, pending: &mut VecDeque<Vec<u8>>) -> Result<()> {
    while let Some(bytes) = pending.front() {
        if !peer.send(bytes.clone())? {
            break;
        }
        pending.pop_front();
    }
    Ok(())
}

pub struct Host {
    peer: Peer,
    identity: Identity,
    visit: Visit,
    phase: Phase,
    generation: u64,
    deadline: Option<Instant>,
    pending: VecDeque<Vec<u8>>,
    local_ready: bool,
    guest_ready: bool,
    input: RemoteInput,
}
impl Host {
    pub fn new(peer: Peer, identity: Identity, visit: Visit) -> Result<Self> {
        identity.compatible(&identity)?;
        visit.validate()?;
        Ok(Self {
            peer,
            identity,
            visit,
            phase: Phase::Waiting,
            generation: 0,
            deadline: None,
            pending: VecDeque::new(),
            local_ready: false,
            guest_ready: false,
            input: RemoteInput::default(),
        })
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn visit(&self) -> &Visit {
        &self.visit
    }
    pub fn acknowledged_input(&self) -> u64 {
        self.input.sequence()
    }
    pub fn send_frame(&self, frame: super::world::Frame) -> Result<bool> {
        if self.phase != Phase::Playing || !self.pending.is_empty() {
            return Ok(false);
        }
        ensure!(
            frame.epoch == self.visit.epoch,
            "Host snapshot belongs to another visit."
        );
        frame.validate()?;
        self.peer.send(super::codec::encode(&serde_json::to_vec(&FromHost::Frame {
            frame: Box::new(frame),
        })?)?)
    }

    pub fn poll(&mut self, now: Instant) -> Result<()> {
        let result = self.poll_inner(now);
        if result.is_err() {
            self.peer.disconnect();
            self.phase = Phase::Waiting;
            self.input = RemoteInput::default();
            self.pending.clear();
            self.deadline = None;
        }
        result
    }
    fn poll_inner(&mut self, now: Instant) -> Result<()> {
        if self.peer.state() != State::Connected {
            if self.phase != Phase::Waiting {
                if let State::Disconnected(reason) = self.peer.state() {
                    eprintln!("Co-op guest connection ended: {reason}");
                }
            }
            self.phase = if self.peer.state() == State::Stopped {
                Phase::Ended
            } else {
                Phase::Waiting
            };
            self.input = RemoteInput::default();
            self.guest_ready = false;
            self.pending.clear();
            self.deadline = None;
            return Ok(());
        }
        if self.generation != self.peer.generation() {
            self.generation = self.peer.generation();
            self.phase = Phase::Checking;
            self.guest_ready = false;
            self.input = RemoteInput::default();
            self.pending.clear();
            self.deadline = Some(now + CHECK_TIMEOUT);
            queue(
                &mut self.pending,
                &FromHost::Hello {
                    identity: self.identity.clone(),
                },
            )?;
        }
        if self.phase == Phase::Waiting {
            return Ok(());
        } // rejection is closing
        ensure!(
            self.deadline.is_none_or(|deadline| now < deadline),
            "The guest took too long to join or load the level."
        );
        for _ in 0..4 {
            let Some(bytes) = self.peer.receive() else {
                break;
            };
            ensure!(bytes.len() <= MAX_INPUT, "Guest message exceeds its limit.");
            let message: FromGuest = serde_json::from_slice(&bytes)
                .map_err(|_| anyhow::anyhow!("The guest sent an invalid session message."))?;
            match message {
                FromGuest::Hello { identity } => {
                    ensure!(self.phase == Phase::Checking, "Unexpected guest handshake.");
                    self.identity.compatible(&identity)?;
                    self.phase = Phase::Loading;
                    self.deadline = Some(now + LOAD_TIMEOUT);
                    queue(
                        &mut self.pending,
                        &FromHost::Enter {
                            visit: self.visit.clone(),
                        },
                    )?;
                }
                FromGuest::Ready { epoch } => {
                    ensure!(
                        matches!(self.phase, Phase::Loading | Phase::Playing),
                        "Guest has not completed the handshake."
                    );
                    ensure!(
                        epoch <= self.visit.epoch,
                        "Guest acknowledged a future level."
                    );
                    if epoch == self.visit.epoch {
                        self.guest_ready = true;
                    }
                }
                FromGuest::Input {
                    epoch,
                    sequence,
                    controls,
                } => {
                    ensure!(
                        matches!(self.phase, Phase::Loading | Phase::Playing),
                        "Guest input arrived before identification."
                    );
                    ensure!(
                        epoch <= self.visit.epoch,
                        "Guest controls refer to a future level."
                    );
                    // In-flight input from a departing visit is ordinary, not an error.
                    if epoch == self.visit.epoch && self.phase == Phase::Playing {
                        self.input.accept(sequence, controls, now)?;
                    }
                }
            }
        }
        if self.phase == Phase::Loading && self.local_ready && self.guest_ready {
            queue(
                &mut self.pending,
                &FromHost::Playing {
                    epoch: self.visit.epoch,
                },
            )?;
            self.phase = Phase::Playing;
            self.deadline = None;
        }
        flush(&self.peer, &mut self.pending)
    }

    pub fn loaded(&mut self, epoch: u64) -> Result<()> {
        ensure!(epoch == self.visit.epoch, "Host loaded the wrong visit.");
        self.local_ready = true;
        Ok(())
    }

    /// Every transition, checkpoint restore and restart gets a fresh epoch.
    pub fn enter(
        &mut self,
        map: String,
        entry: Option<String>,
        difficulty: crate::powerups::Difficulty,
        now: Instant,
    ) -> Result<()> {
        let visit = Visit {
            epoch: self
                .visit
                .epoch
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("Session visit limit reached."))?,
            map,
            entry,
            difficulty,
            combat_preview: false,
        };
        visit.validate()?;
        self.visit = visit;
        self.local_ready = false;
        self.guest_ready = false;
        self.input = RemoteInput::default();
        if matches!(self.phase, Phase::Loading | Phase::Playing) {
            queue(
                &mut self.pending,
                &FromHost::Enter {
                    visit: self.visit.clone(),
                },
            )?;
            self.phase = Phase::Loading;
            self.deadline = Some(now + LOAD_TIMEOUT);
        }
        Ok(())
    }
    pub fn controls(&mut self, now: Instant) -> (crate::movement::Controls, Input) {
        if self.phase == Phase::Playing
            && self.peer.state() == State::Connected
            && self.generation == self.peer.generation()
        {
            self.input.sample(now)
        } else {
            (Default::default(), Default::default())
        }
    }
}

pub struct Guest {
    peer: Peer,
    identity: Identity,
    phase: Phase,
    deadline: Instant,
    verified: bool,
    visit: Option<Visit>,
    new_visit: bool,
    local_ready: bool,
    sequence: u64,
    frame: Option<super::world::Frame>,
    last_tick: Option<u64>,
    pending: VecDeque<Vec<u8>>,
}
impl Guest {
    pub fn new(peer: Peer, identity: Identity, now: Instant) -> Result<Self> {
        identity.compatible(&identity)?;
        Ok(Self {
            peer,
            identity,
            phase: Phase::Connecting,
            deadline: now + CHECK_TIMEOUT,
            verified: false,
            visit: None,
            new_visit: false,
            local_ready: false,
            sequence: 0,
            frame: None,
            last_tick: None,
            pending: VecDeque::new(),
        })
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn take_visit(&mut self) -> Option<Visit> {
        if self.new_visit {
            self.new_visit = false;
            self.visit.clone()
        } else {
            None
        }
    }
    pub fn take_frame(&mut self) -> Option<super::world::Frame> {
        self.frame.take()
    }
    pub fn poll(&mut self, now: Instant) -> Result<()> {
        if self.phase == Phase::Ended {
            return Ok(());
        }
        let result = self.poll_inner(now);
        if result.is_err() {
            self.peer.stop();
            self.phase = Phase::Ended;
            self.pending.clear();
        }
        result
    }
    fn poll_inner(&mut self, now: Instant) -> Result<()> {
        match self.peer.state() {
            State::Disconnected(reason) => bail!("{reason}"),
            State::Stopped => bail!("The co-op session has ended."),
            State::Connected if self.phase == Phase::Connecting => {
                queue(
                    &mut self.pending,
                    &FromGuest::Hello {
                        identity: self.identity.clone(),
                    },
                )?;
                self.phase = Phase::Checking;
                self.deadline = now + CHECK_TIMEOUT;
            }
            _ => (),
        }
        ensure!(
            self.phase == Phase::Playing || now < self.deadline,
            "The host took too long to connect or load the level."
        );
        for _ in 0..4 {
            let Some(bytes) = self.peer.receive() else {
                break;
            };
            ensure!(
                bytes.len() <= MAX_STATE,
                "Host session message exceeds its limit."
            );
            let decoded = super::codec::decode(&bytes)?;
            let message: FromHost = serde_json::from_slice(&decoded)
                .map_err(|_| anyhow::anyhow!("The host sent an invalid session message."))?;
            match message {
                FromHost::Hello { identity } => {
                    ensure!(
                        self.phase == Phase::Checking && !self.verified,
                        "Unexpected host handshake."
                    );
                    self.identity.compatible(&identity)?;
                    self.verified = true;
                }
                FromHost::Enter { visit } => {
                    ensure!(self.verified, "Host has not completed the handshake.");
                    visit.validate()?;
                    ensure!(
                        self.visit
                            .as_ref()
                            .is_none_or(|old| visit.epoch > old.epoch),
                        "Host sent an old level transition."
                    );
                    self.visit = Some(visit);
                    self.new_visit = true;
                    self.local_ready = false;
                    self.phase = Phase::Loading;
                    self.deadline = now + LOAD_TIMEOUT;
                    self.sequence = 0;
                    self.frame = None;
                    self.last_tick = None;
                    self.pending.clear();
                }
                FromHost::Playing { epoch } => {
                    ensure!(
                        self.phase == Phase::Loading
                            && self.local_ready
                            && self.visit.as_ref().is_some_and(|v| v.epoch == epoch),
                        "Host started before both players loaded the level."
                    );
                    self.phase = Phase::Playing;
                }
                FromHost::Frame { frame } => {
                    ensure!(
                        self.phase == Phase::Playing
                            && self.visit.as_ref().is_some_and(|v| v.epoch == frame.epoch),
                        "Host snapshot arrived outside its active visit."
                    );
                    ensure!(
                        self.last_tick.is_none_or(|tick| frame.tick > tick)
                            && frame.acknowledged_input <= self.sequence,
                        "Host snapshot is out of order."
                    );
                    frame.validate()?;
                    self.last_tick = Some(frame.tick);
                    self.frame = Some(*frame);
                }
            }
        }
        flush(&self.peer, &mut self.pending)
    }
    pub fn loaded(&mut self, epoch: u64) -> Result<()> {
        ensure!(
            self.phase == Phase::Loading
                && !self.local_ready
                && self.visit.as_ref().is_some_and(|v| v.epoch == epoch),
            "Guest loaded the wrong visit."
        );
        queue(&mut self.pending, &FromGuest::Ready { epoch })?;
        self.local_ready = true;
        Ok(())
    }
    /// Backpressure returns false; caller retains the latest held controls and
    /// cumulative edge counts. A successful send alone never moves the player.
    pub fn controls(&mut self, controls: Input) -> Result<bool> {
        if self.phase != Phase::Playing {
            return Ok(false);
        }
        controls.validate()?;
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("Session input limit reached."))?;
        let sent = self.peer.send(serde_json::to_vec(&FromGuest::Input {
            epoch: self.visit.as_ref().unwrap().epoch,
            sequence,
            controls,
        })?)?;
        if sent {
            self.sequence = sequence;
        }
        Ok(sent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coop::transport::{self, Conditions};
    use macroquad::prelude::Vec2;
    use std::thread;

    fn identity() -> Identity {
        Identity::current("0".repeat(64))
    }
    fn visit() -> Visit {
        Visit {
            epoch: 1,
            map: "gvillage".into(),
            entry: None,
            difficulty: Default::default(),
            combat_preview: false,
        }
    }
    fn pair() -> (Host, Guest, super::super::invite::Invite) {
        let delay = Conditions {
            delay_ms: 10,
            jitter_ms: 10,
        };
        let (peer, invite) = transport::host(delay).unwrap();
        let host = Host::new(peer, identity(), visit()).unwrap();
        let guest = Guest::new(
            transport::join(invite.clone(), delay).unwrap(),
            identity(),
            Instant::now(),
        )
        .unwrap();
        (host, guest, invite)
    }
    fn until(host: &mut Host, guest: &mut Guest, done: impl Fn(&Host, &Guest) -> bool) {
        let started = Instant::now();
        loop {
            host.poll(Instant::now()).unwrap();
            guest.poll(Instant::now()).unwrap();
            if done(host, guest) {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "session did not reach expected state"
            );
            thread::sleep(Duration::from_millis(2));
        }
    }
    fn play(host: &mut Host, guest: &mut Guest) {
        until(host, guest, |_, g| g.phase() == Phase::Loading);
        let visit = guest.take_visit().unwrap();
        guest.loaded(visit.epoch).unwrap();
        host.loaded(visit.epoch).unwrap();
        until(host, guest, |h, g| {
            h.phase() == Phase::Playing && g.phase() == Phase::Playing
        });
    }

    #[test]
    fn both_players_must_load_then_only_new_visit_inputs_can_move_the_guest() {
        let (mut host, mut guest, _) = pair();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Loading);
        assert_eq!(guest.take_visit().unwrap().epoch, 1);
        assert!(guest.take_visit().is_none());
        guest.loaded(1).unwrap();
        until(&mut host, &mut guest, |h, _| h.guest_ready);
        assert_eq!(host.phase(), Phase::Loading);
        assert!(!guest
            .controls(Input {
                wish: Vec2::X,
                ..Default::default()
            })
            .unwrap());
        host.loaded(1).unwrap();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Playing);
        assert!(guest
            .controls(Input {
                wish: Vec2::X,
                jumps: 1,
                ..Default::default()
            })
            .unwrap());
        until(&mut host, &mut guest, |h, _| h.input.sequence() == 1);
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::X);
        assert!(!host.controls(Instant::now()).0.jump);

        host.enter("skool1".into(), None, Default::default(), Instant::now())
            .unwrap();
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::ZERO);
        // An old packet already on the wire must not survive loading a new map.
        guest
            .peer
            .send(
                serde_json::to_vec(&FromGuest::Input {
                    epoch: 1,
                    sequence: 2,
                    controls: Input {
                        wish: Vec2::X,
                        jumps: 2,
                        ..Default::default()
                    },
                })
                .unwrap(),
            )
            .unwrap();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Loading);
        assert_eq!(guest.take_visit().unwrap().epoch, 2);
        guest.loaded(2).unwrap();
        host.loaded(2).unwrap();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Playing);
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::ZERO);
        guest
            .controls(Input {
                wish: Vec2::Y,
                ..Default::default()
            })
            .unwrap();
        until(&mut host, &mut guest, |h, _| h.input.sequence() == 1);
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::Y);
    }

    #[test]
    fn mismatch_releases_the_guest_slot_and_a_matching_player_can_rejoin() {
        let (peer, invite) = transport::host(Conditions::default()).unwrap();
        let mut host = Host::new(peer, identity(), visit()).unwrap();
        host.loaded(1).unwrap();
        let mut wrong = identity();
        wrong.game_data = "1".repeat(64);
        let mut guest = Guest::new(
            transport::join(invite.clone(), Conditions::default()).unwrap(),
            wrong,
            Instant::now(),
        )
        .unwrap();
        let started = Instant::now();
        let mut rejected = false;
        loop {
            rejected |= host.poll(Instant::now()).is_err();
            rejected |= guest.poll(Instant::now()).is_err();
            if rejected && host.peer.state() != State::Connected && host.generation > 0 {
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(5));
            thread::sleep(Duration::from_millis(2));
        }
        drop(guest);
        let mut replacement = Guest::new(
            transport::join(invite, Conditions::default()).unwrap(),
            identity(),
            Instant::now(),
        )
        .unwrap();
        until(&mut host, &mut replacement, |_, g| {
            g.phase() == Phase::Loading
        });
        replacement.loaded(1).unwrap();
        until(&mut host, &mut replacement, |_, g| {
            g.phase() == Phase::Playing
        });
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::ZERO);
    }

    #[test]
    fn disconnect_and_loading_timeout_release_controls_and_keep_host_available() {
        let (mut host, mut guest, invite) = pair();
        play(&mut host, &mut guest);
        guest
            .controls(Input {
                wish: Vec2::X,
                primary: true,
                ..Default::default()
            })
            .unwrap();
        until(&mut host, &mut guest, |h, _| h.input.sequence() == 1);
        drop(guest);
        let started = Instant::now();
        while host.phase() != Phase::Waiting {
            host.poll(Instant::now()).unwrap();
            assert!(started.elapsed() < Duration::from_secs(5));
            thread::sleep(Duration::from_millis(2));
        }
        let (motion, buttons) = host.controls(Instant::now());
        assert_eq!(motion.wish, Vec2::ZERO);
        assert!(!buttons.primary);
        let mut guest = Guest::new(
            transport::join(invite, Conditions::default()).unwrap(),
            identity(),
            Instant::now(),
        )
        .unwrap();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Loading);
        assert!(host.poll(Instant::now() + LOAD_TIMEOUT).is_err());
        assert_eq!(host.phase(), Phase::Waiting);
    }

    #[test]
    fn guest_input_moves_only_its_host_body_and_the_guest_receives_that_authoritative_state() {
        use crate::{collision::World, inventory::Stats, movement::EYE_HEIGHT};
        use macroquad::prelude::{vec3, Vec3};
        let (mut host, mut guest, _) = pair();
        play(&mut host, &mut guest);
        let world = World::fixture(&[(vec3(-1000., -1000., -100.), vec3(1000., 1000., 0.))]);
        let mut roster = crate::coop::world::Roster::new(
            &world,
            Vec3::Z * (EYE_HEIGHT + 0.04),
            0.,
            Stats::default(),
        )
        .unwrap();
        let start = roster.players[1].player.feet;
        guest
            .controls(Input {
                wish: Vec2::Y,
                ..Default::default()
            })
            .unwrap();
        until(&mut host, &mut guest, |h, _| h.acknowledged_input() == 1);
        for _ in 0..120 {
            roster
                .step(&world, [Default::default(), host.controls(Instant::now())])
                .unwrap();
        }
        assert!(host
            .send_frame(roster.frame(1, host.acknowledged_input()))
            .unwrap());
        until(&mut host, &mut guest, |_, g| g.frame.is_some());
        let received = guest.take_frame().unwrap();
        assert!(received.players[0].player.feet.y.abs() < 0.1);
        assert!((received.players[1].player.feet.y - start.y - 104.).abs() < 5.);
        assert_eq!(
            received.players[1].player.feet,
            roster.players[1].player.feet
        );
        assert_eq!(received.acknowledged_input, 1);
    }
}
