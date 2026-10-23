//! Staged adapter checks using the real host arena and authored geometry.
//! These are not a complete route or a separate-client/network playtest.
use super::{arena::Arena, protocol::{Input, Visit}, world::Roster, PlayerId};
use crate::{assets::Assets, movement::{Controls, Player, EYE_HEIGHT}, render::Scene};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;

struct Check {
    scene: Scene,
    arena: Arena,
    roster: Roster,
}
impl Check {
    fn new(assets: &mut Assets) -> Result<Self> {
        let visit = Visit { epoch: 1, map: "pandemonium".into(), entry: None,
            difficulty: Default::default(), combat_preview: false };
        let mut scene = Scene::load_for(assets, &visit.map, visit.difficulty)?;
        let mut arena = Arena::load(assets, &scene.map, &mut scene.world, &visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(&scene.map, None);
        let roster = Roster::new_with(&scene.world, eye, yaw,
            crate::inventory::Stats::for_level(&visit.map, None),
            |at| !arena.stage.interactions.hazardous(at))?;
        arena.reset_characters(&roster.players);
        Ok(Self { scene, arena, roster })
    }
    async fn step(&mut self, movement: [Controls; 2]) -> Result<()> {
        let inputs = std::array::from_fn(|i| (movement[i], Input {
            yaw: self.roster.players[i].yaw, ..Default::default()
        }));
        self.arena.step(&mut self.roster, &self.scene.map, &mut self.scene.world, inputs)?;
        ensure!(self.roster.checkpoint_generation == 0 && self.roster.players.iter().all(|b| b.stats.alive()),
            "Adapter check unexpectedly died or restored a checkpoint");
        if self.roster.tick % 120 == 0 { next_frame().await; }
        Ok(())
    }
    fn place(&mut self, id: PlayerId, feet: Vec3) -> Result<()> {
        let player = Player::spawn(&self.scene.world, feet + Vec3::Z * EYE_HEIGHT)
            .context("Staged adapter point is obstructed")?;
        self.roster.players[id.index()].player = player;
        self.arena.stage.respawned(id);
        self.arena.reset_characters(&self.roster.players);
        Ok(())
    }
    async fn rope(&mut self, first: PlayerId) -> Result<()> {
        ensure!(!self.scene.world.traversal.managed_ropes, "Single-player rope still owns both players");
        let world = &self.scene.world;
        let mut found = None;
        'find: for rope in &world.traversal.ropes {
            for drop in [80., 160., 240., 320.] {
                let feet = rope.anchor - Vec3::Z * (drop + 40.) + Vec3::X * 24.;
                if drop + 112. > rope.length || !world.body_clear(feet)
                    || !world.body_clear(feet - Vec3::Z * 56. + Vec3::X * 24.) { continue; }
                let mut probe = Player::new(feet);
                probe.tick(world, Controls { use_pressed: true, ..Default::default() });
                if probe.rope.as_ref().is_some_and(|g| g.id == rope.id) {
                    found = Some(feet); break 'find;
                }
            }
        }
        let feet = found.context("No accessible authored Pandemonium rope")?;
        let partner = first.partner();
        self.roster.players[first.index()].player = Player::new(feet);
        // Start outside the automatic-grab reach so the partner's later E edge
        // is a grab, not a release of a grip acquired on the first frame.
        self.roster.players[partner.index()].player = Player::new(feet - Vec3::Z * 56. + Vec3::X * 24.);
        ensure!(world.body_clear(self.roster.players[partner.index()].player.feet), "Second rope staging point obstructed");
        let input = |id: PlayerId, control: Controls| std::array::from_fn(|i| if i == id.index() { control } else { Controls::default() });
        self.step(input(first, Controls { use_pressed: true, ..Default::default() })).await?;
        ensure!(self.roster.players[first.index()].player.rope.is_some(), "First player cannot grab the rope");
        self.step(input(partner, Controls { use_pressed: true, ..Default::default() })).await?;
        ensure!(self.roster.players.iter().all(|b| b.player.rope.is_some()), "Second grab displaced the first grip");
        let before = self.roster.players.each_ref().map(|b| b.player.feet);
        for _ in 0..60 {
            self.step(std::array::from_fn(|i| Controls { rise: if i == first.index() { 1. } else { -1. }, ..Default::default() })).await?;
            for body in &self.roster.players { body.player.validate_world(&self.scene.world)?; }
        }
        ensure!(self.roster.players[first.index()].player.feet.z > before[first.index()].z + 10.
            && self.roster.players[partner.index()].player.feet.z < before[partner.index()].z - 10.,
            "Climbing one rope changed the partner's controls");
        let frame = self.roster.frame(1, 0);
        let decoded: super::world::Frame = serde_json::from_slice(&super::codec::decode(
            &super::codec::encode(&serde_json::to_vec(&frame)?)?)?)?;
        decoded.validate()?;
        for body in &decoded.players { body.player.validate_world(&self.scene.world)?; }
        ensure!(serde_json::to_value(&decoded)? == serde_json::to_value(&frame)?, "Two grips did not survive the wire envelope");
        self.step(input(first, Controls { use_pressed: true, ..Default::default() })).await?;
        ensure!(self.roster.players[first.index()].player.rope.is_none()
            && self.roster.players[partner.index()].player.rope.is_some(), "One release detached the partner");
        println!("PASS Pandemonium {first:?} first: independent grabs, opposing climbs, wire roundtrip and personal release");
        Ok(())
    }
    async fn cart(&mut self, first: PlayerId) -> Result<()> {
        use crate::pandemonium::Cart;
        let partner = first.partner();
        self.place(first, vec3(-5024., 1368., 32.))?;
        self.place(partner, vec3(-5024., 1600., -25.))?;
        for _ in 0..240 {
            self.step([Controls::default(); 2]).await?;
            ensure!(self.arena.stage.interactions.pandemonium.as_ref().unwrap().state.cart == Cart::Waiting
                && self.arena.stage.owner.is_none(), "Minecart departed without its partner");
        }
        ensure!(self.arena.stage.notices[first.index()].as_ref().is_some_and(|n| n.text.contains("board")),
            "Missing shared boarding notice");
        let state = self.arena.snapshot();
        self.arena.replicate(&serde_json::from_slice(&serde_json::to_vec(&state)?)?, &self.scene.map, &mut self.scene.world)?;
        self.place(partner, vec3(-5000., 1368., 32.))?;
        self.step([Controls::default(); 2]).await?;
        ensure!(self.arena.stage.owner == Some(first), "Minecart lost its first waiting requester");
        let waiting = self.roster.players[partner.index()].player.feet;
        for _ in 0..14_400 {
            let mut movement = [Controls::default(); 2];
            movement[first.index()].use_pressed = self.roster.tick % 30 == 0;
            self.step(movement).await?;
            if self.arena.stage.owner.is_none() { break; }
            ensure!(self.arena.stage.owner == Some(first)
                && self.roster.players[partner.index()].player.feet == waiting,
                "Minecart changed its scene owner or moved the waiting partner");
        }
        let p = self.arena.stage.interactions.pandemonium.as_ref().unwrap();
        ensure!(p.state.cart == Cart::Finished && p.state.rides == 1 && self.arena.stage.owner.is_none(),
            "Minecart did not finish exactly once");
        ensure!(self.roster.players[0].player.feet.distance(self.roster.players[1].player.feet) <= 110.
            && self.roster.players[0].warped_at == self.roster.players[1].warped_at
            && self.roster.players[0].warped_at > 0, "Minecart did not safely reunite the pair");
        ensure!(self.roster.players.iter().all(|b| self.scene.world.body_clear(b.player.feet)), "Blocked minecart reunion");
        let key = self.scene.map.entities.iter().find(|e| e.get("targetname").is_some_and(|n| n == "gnome_key"))
            .and_then(|e| e.get("origin")).and_then(|s| crate::interaction::vector(s)).context("Missing key marker")?;
        self.place(partner, key - Vec3::Z * EYE_HEIGHT)?;
        self.step([Controls::default(); 2]).await?;
        ensure!(self.arena.stage.interactions.pandemonium.as_ref().unwrap().state.key, "Partner cannot collect shared quest key");
        println!("PASS Pandemonium {first:?} first: wait for both, retained requester, one authored cart ride, safe reunion and partner key pickup");
        Ok(())
    }
}

pub async fn check(assets: &mut Assets) -> Result<()> {
    for first in PlayerId::ALL {
        Check::new(assets)?.rope(first).await?;
        Check::new(assets)?.cart(first).await?;
    }
    println!("PASS staged Pandemonium co-op adapters / build {} / full traversal and public clients remain unverified", env!("LOOKING_GLASS_BUILD"));
    Ok(())
}
