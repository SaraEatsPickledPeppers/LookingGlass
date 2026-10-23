//! Collision-checked partner placement, including authored slopes and swimming.
use crate::{
    collision::World,
    movement::{Player, EYE_HEIGHT},
    water::{Immersion, Liquid},
};
use macroquad::prelude::*;

/// Entrance markers may be above their playable landing. Only arrival may
/// search that far down; an ordinary revival must remain near the survivor.
pub fn entrance(world: &World, feet: Vec3, safe: impl Fn(Vec3) -> bool) -> Option<Player> {
    near(world, feet, 256., safe)
}

pub fn near(world: &World, feet: Vec3, drop: f32, safe: impl Fn(Vec3) -> bool) -> Option<Player> {
    let source = Immersion::sample(world, feet);
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
            let beside = feet + direction * radius;
            // A swimmer needs clear water at the same depth, not a dry floor.
            // Trace against walls and sample water along the route, rather than
            // accepting only an endpoint on the far side of a shore.
            if source.kind == Liquid::Water
                && source.level >= 2
                && safe(beside)
                && clear_path(world, feet, beside)
                && (0..=8).all(|i| {
                    let at = feet.lerp(beside, i as f32 / 8.);
                    let liquid = Immersion::sample(world, at);
                    liquid.kind == Liquid::Water && liquid.level >= 2
                })
            {
                return Player::spawn(world, beside + Vec3::Z * EYE_HEIGHT);
            }
            let candidate = beside + Vec3::Z * 18.;
            if !clear_path(world, feet + Vec3::Z, candidate) {
                continue;
            }
            let distance = drop + 18.;
            let floor = world.body_trace(candidate, candidate - Vec3::Z * distance);
            if floor.start_solid || floor.fraction >= 1. || floor.normal.z < 0.65 {
                continue;
            }
            let landing = candidate - Vec3::Z * (distance * floor.fraction);
            if !supported(world, landing)
                || !safe(landing)
                || !clear_path(world, candidate, landing)
            {
                continue;
            }
            return Player::spawn(world, landing + Vec3::Z * EYE_HEIGHT);
        }
    }
    None
}

fn clear_path(world: &World, from: Vec3, to: Vec3) -> bool {
    let hit = world.body_trace(from, to);
    !hit.start_solid && hit.fraction >= 1. && world.body_clear(to)
}

/// Require a full footprint plus a small edge margin. Probe around the actual
/// floor plane: a fixed four-unit downward ray incorrectly rejects normal hills.
fn supported(world: &World, feet: Vec3) -> bool {
    if !world.body_clear(feet) || Immersion::sample(world, feet).level != 0 {
        return false;
    }
    let start = feet + Vec3::Z;
    let center = world.sweep(start, start - Vec3::Z * 41., Vec3::ZERO);
    if center.start_solid || center.fraction >= 1. || center.normal.z < 0.65 {
        return false;
    }
    let ground = start - Vec3::Z * (41. * center.fraction);
    [-20., 0., 20.].into_iter().all(|x| {
        [-20., 0., 20.].into_iter().all(|y| {
            let expected = ground
                + vec3(
                    x,
                    y,
                    -(center.normal.x * x + center.normal.y * y) / center.normal.z,
                );
            let from = expected + Vec3::Z * 8.;
            let hit = world.sweep(from, from - Vec3::Z * 16., Vec3::ZERO);
            !hit.start_solid && hit.fraction < 1. && hit.normal.z >= 0.65
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swimmer_revives_at_the_same_depth_with_fresh_air_but_never_in_harmful_liquid() {
        use crate::collision::{Collider, Liquid as Volume};
        let mut world = World::fixture(&[]);
        let volume = |contents| Volume {
            contents,
            volume: Collider::fixture(Vec3::splat(-500.), Vec3::splat(500.)),
        };
        world.set_dynamic_liquids(vec![volume(32)]);
        let player = near(&world, Vec3::ZERO, 32., |_| true).unwrap();
        assert!(player.swimming && !player.grounded);
        assert_eq!(player.feet.z, 0.);
        assert_eq!(player.breath.submerged, 0.);
        assert!(near(&world, Vec3::ZERO, 32., |_| false).is_none());
        for harmful in [8, 16] {
            world.set_dynamic_liquids(vec![volume(harmful)]);
            assert!(near(&world, Vec3::ZERO, 32., |_| true).is_none());
        }
        // Even if a distant water volume contains the endpoint, do not cross
        // an intervening dry gap while reviving an underwater player.
        world.set_dynamic_liquids(vec![
            Volume {
                contents: 32,
                volume: Collider::fixture(Vec3::splat(-10.), vec3(10., 10., 100.)),
            },
            Volume {
                contents: 32,
                volume: Collider::fixture(vec3(40., -100., -100.), vec3(100., 100., 100.)),
            },
        ]);
        assert!(near(&world, Vec3::ZERO, 32., |_| true).is_none());
    }
    #[test]
    fn elevated_entrance_can_land_but_respawn_cannot_drop_that_far() {
        let world = World::fixture(&[(vec3(-500., -500., -100.), vec3(500., 500., 0.))]);
        let feet = Vec3::Z * 232.;
        assert!(near(&world, feet, 32., |_| true).is_none());
        let placed = entrance(&world, feet, |_| true).unwrap();
        assert!(placed.grounded && placed.feet.z.abs() < 0.1);
        assert!(entrance(&world, Vec3::Z * 300., |_| true).is_none());
        assert!(entrance(&world, feet, |at| at.z > 10.).is_none());
    }
    #[test]
    fn entrance_does_not_cross_a_wall_or_accept_a_narrow_ledge() {
        let ledge = World::fixture(&[(vec3(-500., -12., -100.), vec3(500., 12., 0.))]);
        assert!(entrance(&ledge, Vec3::Z * 0.04, |_| true).is_none());
        let world = World::fixture(&[
            (vec3(-20., -20., -100.), vec3(20., 20., 0.)),
            (vec3(30., -500., -100.), vec3(500., 500., 0.)),
            (vec3(22., -500., -10.), vec3(28., 500., 100.)),
        ]);
        assert!(entrance(&world, Vec3::Z * 0.04, |_| true).is_none());
    }
}
