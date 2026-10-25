//! Focused native return-quest gates, using the production paired stage.
//! Bodies are staged at authored contacts; this is not a traversal/network test.
use super::{protocol::Visit, stage::Stage, world::Roster, PlayerId};
use crate::{assets::Assets, bsp::Bsp, collision::World, inventory::Stats,
    movement::{Player, EYE_HEIGHT, FIXED_DT}, school_return::Phase};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;

pub(super) fn check(assets: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&assets.read("maps/skool1.bsp")?)?;
    let visit = Visit { epoch: 1, map: "skool1".into(), entry: Some("skool1_start2".into()),
        difficulty: Default::default(), combat_preview: false };
    for first in PlayerId::ALL {
        for holder in PlayerId::ALL {
            for dead_partner in [false, true] {
                let mut world = World::from_bsp(&map)?;
                let mut stage = Stage::load(assets, &map, &mut world, &visit, false)?;
                let (eye, yaw) = crate::interaction::spawn(&map, visit.entry.as_deref());
                let mut roster = Roster::new(&world, eye, yaw, Stats::for_level("skool1", visit.entry.as_deref()))?;
                for body in &mut roster.players { body.stats.school_items = Default::default(); }
                roster.players[holder.index()].stats.school_items.star = true;
                roster.players[holder.partner().index()].stats.school_items.potion = true;
                roster.players[0].stats.damage(13.);
                roster.players[1].stats.damage(27.);
                if dead_partner { roster.players[first.partner().index()].stats.damage(1000.); }
                let mut expected = roster.players.each_ref().map(|b| b.stats.clone());
                let door = vec3(692., 4174., 384.03125);
                roster.players[first.index()].player = Player::spawn(&world, door + Vec3::Z * EYE_HEIGHT)
                    .context("Cannot stage return star-door contact")?;
                stage.before_move(&mut roster, &map, &mut world, &Default::default(), FIXED_DT)?;
                let before = roster.players.each_ref().map(|b| b.player.feet);
                stage.contacts(&mut roster, &world, before)?;
                let state = stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap();
                ensure!(state.phase == Phase::Open && !state.star && state.potion,
                    "Carried return rewards did not open star doors: first {first:?}, holder {holder:?}, dead partner {dead_partner}");
                for stats in &mut expected { stats.school_items.star = false; }
                ensure!(serde_json::to_value(roster.players.each_ref().map(|b| &b.stats))?
                    == serde_json::to_value(&expected)?, "Return star gate changed personal resources or left spent copies");
                println!("PASS co-op return star gate first {first:?}, reward holder {holder:?}, dead partner {dead_partner}: shared quest, personal resources preserved");
                // Stage only the later prerequisite/position to isolate the
                // authored potion contact and performance. The lift and actual
                // route to this walkway require separate traversal evidence.
                stage.interactions.school.as_mut().unwrap().return_visit.as_mut().unwrap().enter(Phase::Observatory);
                let exit = vec3(340., 4396., 1344.03125);
                roster.players[first.index()].player = Player::spawn(&world, exit + Vec3::Z * EYE_HEIGHT)
                    .context("Cannot stage observatory exit contact")?;
                stage.before_move(&mut roster, &map, &mut world, &Default::default(), FIXED_DT)?;
                let before = roster.players.each_ref().map(|b| b.player.feet);
                stage.contacts(&mut roster, &world, before)?;
                ensure!(stage.owner == Some(first), "First arrival did not own the observatory exit");
                let waiting = roster.players[first.partner().index()].player.feet;
                for _ in 0..120*20 {
                    if stage.transition.is_some() { break; }
                    stage.scene_step(&mut roster, &map, &mut world, &Default::default())?;
                    ensure!(stage.owner == Some(first)
                        && roster.players[first.partner().index()].player.feet == waiting,
                        "Observatory performance moved its waiting partner or changed owner");
                    let phase = stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap().phase;
                    for (body, old) in roster.players.iter().zip(&expected) {
                        ensure!(body.stats.school_items.potion == (old.school_items.potion && phase < Phase::Drinking),
                            "Potion was consumed at the wrong scene phase");
                    }
                }
                ensure!(stage.transition == Some(("potears1".into(), Some("potears1_start1".into()))),
                    "Observatory did not request the authored Pool of Tears entrance: phase {:?}, owner {:?}",
                    stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap().phase, stage.owner);
                for stats in &mut expected { stats.school_items.potion = false; }
                ensure!(serde_json::to_value(roster.players.each_ref().map(|b| &b.stats))?
                    == serde_json::to_value(&expected)?, "Potion scene changed personal resources or left spent copies");
                println!("PASS co-op return potion first {first:?}, reward holder {holder:?}, dead partner {dead_partner}: original scene, frozen partner, phase consumption and authored exit");
            }
        }
    }
    Ok(())
}
