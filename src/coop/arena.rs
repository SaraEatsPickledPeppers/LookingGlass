//! Host-owned native combat. Campaign controllers join this once-per-tick
//! pipeline later; clients only restore presentation, never run enemy AI.
use super::{
    protocol::{Input, Visit},
    world::{Body, Roster},
    PlayerId,
};
use crate::{
    assets::Assets,
    bsp::Bsp,
    character::Character,
    collision::World,
    combat,
    inventory::{self, Catalog, Pickup},
    movement::{Controls, FIXED_DT},
    npc::Npcs,
};
use anyhow::Result;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub npcs: crate::npc::Snapshot,
    pub characters: [crate::character::Snapshot; 2],
}

pub struct Arena {
    pub characters: [Character; 2],
    pub npcs: Npcs,
    pub pickups: Vec<Pickup>,
    pub catalog: Catalog,
    pub visit_key: String,
    initial: crate::npc::Snapshot,
    alive: [bool; 2],
}
impl Arena {
    pub fn load(assets: &mut Assets, map: &Bsp, visit: &Visit, staged: bool) -> Result<Self> {
        let npcs = Npcs::load(
            assets,
            map,
            &visit.map,
            visit.entry.as_deref(),
            false,
            staged,
        )?;
        let initial = npcs.snapshot();
        let catalog = Catalog::load(assets)?;
        let pickups =
            inventory::pickups_for_visit(map, &visit.map, visit.entry.as_deref(), &catalog);
        Ok(Self {
            npcs,
            initial,
            catalog,
            pickups,
            visit_key: crate::save::visit_key(&visit.map, visit.entry.as_deref()),
            characters: [
                Character::load_for_player(assets, PlayerId::Blue)?,
                Character::load_for_player(assets, PlayerId::Red)?,
            ],
            alive: [true; 2],
        })
    }
    pub fn reset_characters(&mut self, bodies: &[Body; 2]) {
        for (character, body) in self.characters.iter_mut().zip(bodies) {
            character.reset(&body.player, body.yaw);
        }
        self.alive = bodies.each_ref().map(|p| p.stats.alive());
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            npcs: self.npcs.snapshot(),
            characters: self.characters.each_ref().map(|c| c.snapshot()),
        }
    }
    pub fn replicate(&mut self, s: &Snapshot) -> Result<()> {
        self.npcs.replicate(&s.npcs)?;
        for (c, state) in self.characters.iter_mut().zip(&s.characters) {
            c.restore(state)?;
        }
        Ok(())
    }
    pub fn step(
        &mut self,
        roster: &mut Roster,
        world: &World,
        mut inputs: [(Controls, Input); 2],
    ) -> Result<()> {
        for (character, (movement, _)) in self.characters.iter().zip(&mut inputs) {
            if character.attack_movement_locked() || character.ice_locked() {
                *movement = Controls::default();
            }
        }
        roster.move_players(world, inputs);
        let before = self.npcs.loot_sources();
        // One clock for the world. Either player's Watch can stop its enemies;
        // each player's own movement, weapon and resource clocks still run.
        let world_dt = roster
            .players
            .iter()
            .fold(FIXED_DT, |dt, b| b.stats.powers.world_dt(dt));
        for index in 0..2 {
            let character = &mut self.characters[index];
            let body = &mut roster.players[index];
            if body.stats.alive() {
                if let Some(weapon) = inputs[index].1.weapon {
                    body.stats.select(weapon as usize);
                }
                inventory::collect(&mut body.stats, &self.pickups, body.player.feet, world);
            } else {
                character.die();
            }
            let mut targets = self.npcs.targets();
            targets.extend(character.ice_targets());
            if let Some(summon) = character.summon_target() {
                targets.push(summon);
            }
            character.world_time(FIXED_DT, world_dt);
            character.notarget(body.stats.ignores_alice());
            let input = inputs[index].1;
            let click = if !body.stats.alive() {
                None
            } else if input.primary {
                Some(false)
            } else if input.alternate {
                Some(true)
            } else {
                None
            };
            character.update_funded(
                FIXED_DT,
                &body.player,
                body.running,
                crate::weapons::WeaponInput {
                    dice: body.stats.copies(6),
                    selected: body.stats.equipped().unwrap_or(crate::weapons::UNARMED),
                    click,
                    aim: direction(body.yaw, body.pitch),
                    first_person: false,
                },
                &combat::Context {
                    world,
                    targets: &targets,
                },
                Some(&mut body.stats),
            );
            for mut hit in character.take_hits() {
                if character.hit_ice(hit) {
                    continue;
                }
                if hit.id == crate::dice::ALICE {
                    body.stats.damage(hit.damage);
                    body.player.knockback(hit.knockback * 2.5);
                } else if hit.id == crate::dice::SUMMON {
                    character.hit_summon(hit);
                } else {
                    if hit.kind.is_demon() {
                        hit.kind = hit.kind.means();
                    } else {
                        hit.damage = body.stats.attack_damage(hit.damage);
                    }
                    self.npcs.hit(hit);
                }
            }
            // These transient queues are consumed even when sound is disabled.
            character.take_audio();
            character.take_world_audio();
        }
        let targets = roster.players.each_ref().map(|body| combat::PlayerTarget {
            eye: body.player.eye(),
            alive: body.stats.alive(),
            hidden: body.stats.ignores_alice(),
        });
        let feedback = self.npcs.update_players(world_dt, world, targets);
        for (body, hit) in roster.players.iter_mut().zip(feedback.players) {
            body.stats.damage(hit.damage);
            body.player.knockback(hit.impulse);
            if hit.will_drain > 0. {
                body.stats.spend_will(hit.will_drain.min(body.stats.will()));
            }
        }
        let after = self.npcs.loot_sources();
        for body in &mut roster.players {
            crate::loot::tick(
                &mut body.stats,
                &self.visit_key,
                world_dt,
                &before,
                &after,
                body.player.feet,
                world,
            );
        }
        let generation = roster.checkpoint_generation;
        roster.finish_tick(world)?;
        if generation != roster.checkpoint_generation {
            self.npcs.restore(&self.initial)?;
            self.reset_characters(&roster.players);
        } else {
            for (index, (character, body)) in
                self.characters.iter_mut().zip(&roster.players).enumerate()
            {
                let alive = body.stats.alive();
                if alive && !self.alive[index] {
                    character.reset(&body.player, body.yaw);
                }
                if !alive {
                    character.die();
                }
                self.alive[index] = alive;
            }
        }
        Ok(())
    }
}

pub fn direction(yaw: f32, pitch: f32) -> Vec3 {
    vec3(
        yaw.cos() * pitch.cos(),
        yaw.sin() * pitch.cos(),
        pitch.sin(),
    )
}
