//! The lamps as light, and the dim ones fixed in the maze. See
//! `specs/0002-walking-it.md`.

use blitzkit::lighting::{PointLight, MAX_POINT_LIGHTS};
use glam::{vec3, Vec3};

use crate::lamps::Lamps;
use crate::maze::{self, Maze};
use crate::walls::{cell_centre, TALL};

/// How far a lamp reaches. The number that decides whether this is a game: far
/// enough and you never put one down, short enough and you walk in a bubble.
pub const LAMP_RANGE: f32 = 9.0;
pub const LAMP_COLOR: Vec3 = vec3(1.0, 0.86, 0.62);
pub const LAMP_INTENSITY: f32 = 0.5;

/// The dim ones, which do not cast. Few, and weak enough that a corridor is
/// worth lighting.
pub const FIXED_RANGE: f32 = 7.0;
pub const FIXED_COLOR: Vec3 = vec3(0.42, 0.48, 0.70);
pub const FIXED_INTENSITY: f32 = 0.22;

/// How high a lamp sits off the floor, and how big the thing itself is.
pub const STANDS: f32 = 0.7;
const HEIGHT: f32 = STANDS;

/// Where the fixed lights are: spread through the maze by index, so they do not
/// move between runs of the same maze.
///
/// Six, leaving room for the two lamps inside spec 0020's eight. A ninth light
/// is not an error, it is a lamp that quietly stops working.
pub fn fixed(maze: &Maze) -> Vec<PointLight> {
    let count = MAX_POINT_LIGHTS - crate::lamps::LAMPS;

    // never at a dead end: a light down a stub lights a place nobody needs to
    // see, and the point of these is to make the through routes walkable
    let ends = maze.dead_ends();
    let open: Vec<usize> = (0..maze::CELLS).filter(|c| !ends.contains(c)).collect();
    let step = open.len() / (count + 1);

    (1..=count)
        .map(|n| {
            let cell = open[(n * step) % open.len()];
            let at = cell_centre(cell) + Vec3::Y * TALL * 0.75;
            PointLight::new(at, FIXED_COLOR, FIXED_INTENSITY, FIXED_RANGE)
        })
        .collect()
}

/// How far out in front a carried lamp is held, and how far to each side when
/// both are. At the eye it would be a light the camera is standing inside.
pub const HELD_OUT: f32 = 1.15;
pub const HELD_DOWN: f32 = 0.62;
const HELD_ASIDE: f32 = 0.42;

/// Where the lamps in hand are: held out in front, and one to each side when
/// both are, so two in hand are not one light of twice the strength.
///
/// The drawing wants these as well as the lighting, and a lamp drawn somewhere
/// its light is not would be worse than not drawing it at all.
pub fn carried_at(lamps: &Lamps, hand: Vec3, aside: Vec3) -> Vec<Vec3> {
    let carried = lamps.carried();

    (0..carried)
        .map(|n| {
            let across = if carried > 1 {
                aside * if n == 0 { -HELD_ASIDE } else { HELD_ASIDE }
            } else {
                Vec3::ZERO
            };
            hand + across
        })
        .collect()
}

/// Every light the scene should carry: the fixed ones, then the two lamps.
///
/// The lamps go last so that if this ever exceeds eight it is a fixed light
/// that is dropped rather than the one in your hand.
pub fn all(maze: &Maze, lamps: &Lamps, hand: Vec3, aside: Vec3) -> Vec<PointLight> {
    let mut lights = fixed(maze);

    for cell in lamps.standing() {
        lights.push(
            PointLight::new(
                cell_centre(cell) + Vec3::Y * HEIGHT,
                LAMP_COLOR,
                LAMP_INTENSITY,
                LAMP_RANGE,
            )
            .casting(),
        );
    }

    for at in carried_at(lamps, hand, aside) {
        lights.push(PointLight::new(at, LAMP_COLOR, LAMP_INTENSITY, LAMP_RANGE).casting());
    }

    lights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lamps::LAMPS;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn maze() -> Maze {
        Maze::carve(&mut StdRng::seed_from_u64(3))
    }

    #[test]
    fn a_standing_lamp_lights_its_cell() {
        let maze = maze();
        let mut lamps = Lamps::new();
        let cell = maze::index(5, 5);
        lamps.put_down(cell);

        let here = cell_centre(cell);
        let lit = all(&maze, &lamps, Vec3::ZERO, Vec3::X)
            .into_iter()
            .filter(|light| light.casts)
            .any(|light| (light.position - here).length() < 1.0);

        assert!(lit, "nothing casts where the lamp was put down");
    }

    #[test]
    fn a_carried_lamp_follows_you() {
        let maze = maze();
        let lamps = Lamps::new();
        let standing_at = vec3(12.0, 0.0, -7.0);

        let lit = all(&maze, &lamps, standing_at, Vec3::X)
            .into_iter()
            .filter(|light| light.casts)
            .all(|light| (light.position - standing_at).length() < 1.0);

        assert!(lit, "a lamp in hand is somewhere else");
    }

    #[test]
    fn only_the_two_cast() {
        let maze = maze();
        let mut lamps = Lamps::new();
        lamps.put_down(maze::index(2, 2));

        let casting = all(&maze, &lamps, Vec3::ZERO, Vec3::X)
            .into_iter()
            .filter(|light| light.casts)
            .count();

        assert_eq!(casting, LAMPS, "the fixed ones must not cast");
    }

    #[test]
    fn there_are_never_more_than_eight() {
        // spec 0020 takes eight and drops the rest, silently
        let maze = maze();
        let mut lamps = Lamps::new();

        for put_down in 0..=LAMPS {
            assert!(
                all(&maze, &lamps, Vec3::ZERO, Vec3::X).len() <= MAX_POINT_LIGHTS,
                "with {} put down",
                put_down
            );
            lamps.put_down(maze::index(put_down, 0));
        }
    }

    #[test]
    fn no_fixed_light_is_down_a_dead_end() {
        let maze = maze();
        let ends = maze.dead_ends();

        for light in fixed(&maze) {
            let at = (0..maze::CELLS)
                .min_by(|a, b| {
                    let d = |c: &usize| (cell_centre(*c) - light.position).length();
                    d(a).total_cmp(&d(b))
                })
                .expect("cells exist");

            assert!(!ends.contains(&at), "a light sits at a dead end");
        }
    }

    #[test]
    fn the_fixed_lights_are_in_the_maze() {
        let maze = maze();
        let reach = maze::WIDTH as f32 * crate::walls::CELL;

        for light in fixed(&maze) {
            assert!(
                light.position.x.abs() < reach && light.position.z.abs() < reach,
                "a fixed light is outside at {:?}",
                light.position
            );
        }
    }
}
