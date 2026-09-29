//! The lamps as light, and the dim ones fixed in the maze. See
//! `specs/0002-walking-it.md`.

use blitzkit::collision::{sweep_sphere, Aabb, Sphere};
use blitzkit::lighting::{PointLight, MAX_POINT_LIGHTS};
use glam::{vec3, Vec3};

use crate::lamps::Lamps;
use crate::maze::{self, Maze};
use crate::walls::cell_centre;

/// How far a lamp reaches. The number that decides whether this is a game: far
/// enough and you never put one down, short enough and you walk in a bubble.
pub const LAMP_RANGE: f32 = 9.0;
pub const LAMP_COLOR: Vec3 = vec3(1.0, 0.86, 0.62);
pub const LAMP_INTENSITY: f32 = 0.5;

/// The dim ones, which do not cast, and so cannot be hidden by a wall. The only
/// thing that keeps one out of the corridor next door is its range running out
/// before it gets there.
///
/// From a cell centre the near face of a wall is half a cell away and the far
/// face is half a cell plus its thickness, so anything under that lights its own
/// cell and nothing beyond it. At seven it lit 123 cells, 92 of them through
/// stone, which is the glow that looked like a bug.
pub const FIXED_RANGE: f32 = 1.55;
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
pub fn fixed_cells(maze: &Maze) -> Vec<usize> {
    let count = MAX_POINT_LIGHTS - crate::lamps::LAMPS;

    // never at a dead end: a light down a stub lights a place nobody needs to
    // see, and the point of these is to make the through routes walkable
    // nor where you stand at the start or finish: something solid in the cell
    // you spawn in is something you spawn inside
    let ends = maze.dead_ends();
    let open: Vec<usize> = (0..maze::CELLS)
        .filter(|c| !ends.contains(c) && *c != maze.start && *c != maze.exit)
        .collect();
    let step = open.len() / (count + 1);

    (1..=count).map(|n| open[(n * step) % open.len()]).collect()
}

/// Where a brazier's flame stands in a cell, which is where its light is.
///
/// The centre, and only the centre. It is the one place equidistant from all
/// four walls, so it is the only place a range cutoff can hold the light in.
/// Anything against a wall has that wall at no distance at all and shines
/// through it whatever the range.
pub fn brazier_at(cell: usize) -> Vec3 {
    cell_centre(cell) + Vec3::Y * crate::brazier::FLAME_AT
}

pub fn fixed(maze: &Maze) -> Vec<PointLight> {
    fixed_cells(maze)
        .into_iter()
        .map(|cell| PointLight::new(brazier_at(cell), FIXED_COLOR, FIXED_INTENSITY, FIXED_RANGE))
        .collect()
}

/// How far out in front a carried lamp is held, and how far to each side when
/// both are. At the eye it would be a light the camera is standing inside.
pub const HELD_OUT: f32 = 1.15;
pub const HELD_DOWN: f32 = 0.62;
const HELD_ASIDE: f32 = 0.42;

/// How much room a flame wants from a wall, so it does not sit inside one.
const FLAME_CLEARANCE: f32 = 0.22;

/// And a little more, so it stops short of the wall rather than against it.
const SKIN: f32 = 0.04;

/// Where the candles in hand are: held out in front, one to each side when both
/// are, and each stopped by whatever wall is in its own way.
///
/// Each is swept separately on purpose. Sweeping the hand and then stepping the
/// candles out to either side of it puts the sideways step after the test, so a
/// candle clear at the hand can still end up through a wall.
pub fn carried_at(
    lamps: &Lamps,
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    walls: &[Aabb],
) -> Vec<Vec3> {
    let carried = lamps.carried();

    (0..carried)
        .map(|n| {
            let across = if carried > 1 {
                right * if n == 0 { -HELD_ASIDE } else { HELD_ASIDE }
            } else {
                Vec3::ZERO
            };
            let reach = forward * HELD_OUT - Vec3::Y * HELD_DOWN + across;

            eye + reach * clear_along(eye, reach, walls)
        })
        .collect()
}

/// How much of a reach is clear of every wall, from none of it to all of it.
///
/// `sweep_sphere` reports how far it got in world units, not what fraction of
/// the movement that was. Reading it as a fraction is what let a candle through
/// a wall: the reach is longer than one unit, so a hit at 1.2 clamped to 1.0
/// and the candle was placed the whole 1.38 out, past the wall it just hit.
fn clear_along(eye: Vec3, reach: Vec3, walls: &[Aabb]) -> f32 {
    let body = Sphere::new(eye, FLAME_CLEARANCE);
    let length = reach.length();

    if length < f32::EPSILON {
        return 0.0;
    }

    let nearest = walls
        .iter()
        .filter_map(|wall| sweep_sphere(&body, reach, wall))
        .map(|hit| hit.distance)
        .fold(f32::INFINITY, f32::min);

    ((nearest - SKIN) / length).clamp(0.0, 1.0)
}

/// Every light the scene should carry: the fixed ones, then the two lamps.
///
/// The lamps go last so that if this ever exceeds eight it is a fixed light
/// that is dropped rather than the one in your hand.
pub fn all(
    maze: &Maze,
    lamps: &Lamps,
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    walls: &[Aabb],
) -> Vec<PointLight> {
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

    for at in carried_at(lamps, eye, forward, right, walls) {
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
    fn a_wall_stops_the_candle_reaching_through_it() {
        let eye = Vec3::new(0.0, 1.5, 0.0);
        let wall = Aabb::from_center_size(Vec3::new(0.0, 1.0, -0.6), Vec3::new(8.0, 3.0, 0.3));
        let lamps = Lamps::new();

        for at in carried_at(&lamps, eye, -Vec3::Z, Vec3::X, &[wall]) {
            assert!(at.z > -0.6, "a candle is through the wall at {}", at.z);
        }
    }

    #[test]
    fn every_candle_is_swept_not_just_the_hand() {
        // a wall to one side only: the candle on that side must be pulled in
        // and the other must not be. Sweeping the hand alone cannot do this.
        let eye = Vec3::new(0.0, 1.5, 0.0);
        let wall = Aabb::from_center_size(Vec3::new(0.5, 1.0, -0.6), Vec3::new(0.3, 3.0, 3.0));
        let lamps = Lamps::new();

        let held = carried_at(&lamps, eye, -Vec3::Z, Vec3::X, &[wall]);
        let reach: Vec<f32> = held.iter().map(|at| (*at - eye).length()).collect();

        assert_eq!(reach.len(), 2);
        assert!(
            (reach[0] - reach[1]).abs() > 0.1,
            "both were held the same: {:?}",
            reach
        );
    }

    #[test]
    fn how_far_it_got_is_not_how_much_of_the_reach_that_was() {
        // the reach is longer than one unit, so a wall hit at 1.2 is most of
        // the way out, not all of it and then some
        let eye = Vec3::new(0.0, 1.5, 0.0);
        let face = -1.30;
        let wall =
            Aabb::from_center_size(Vec3::new(0.0, 1.0, face - 0.15), Vec3::new(8.0, 3.0, 0.3));
        let lamps = Lamps::new();

        for at in carried_at(&lamps, eye, -Vec3::Z, Vec3::X, &[wall]) {
            assert!(
                at.z > face,
                "a candle reached {} and the wall face is at {}",
                at.z,
                face
            );
        }
    }

    #[test]
    fn no_candle_ends_up_inside_a_wall_anywhere_in_the_maze() {
        // walk the whole maze, look every way, and check both candles
        let maze = maze();
        let walls = crate::walls::colliders(&maze);
        let lamps = Lamps::new();

        for cell in 0..maze::CELLS {
            for shift in [-0.6f32, -0.3, 0.0, 0.3, 0.6] {
                for step in 0..16 {
                    let yaw = step as f32 / 16.0 * std::f32::consts::TAU;
                    let (sin, cos) = yaw.sin_cos();
                    let forward = vec3(sin, 0.0, -cos);
                    let right = forward.cross(Vec3::Y);
                    let eye = cell_centre(cell) + forward * shift + Vec3::Y * crate::player::EYE;

                    for at in carried_at(&lamps, eye, forward, right, &walls) {
                        for wall in &walls {
                            assert!(
                                !wall.contains_point(at),
                                "a candle is inside a wall at {:?}, cell {}, yaw {}",
                                at,
                                cell,
                                yaw
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn nothing_in_the_way_holds_it_at_arms_length() {
        let eye = Vec3::new(0.0, 1.5, 0.0);
        let mut lamps = Lamps::new();
        lamps.put_down(0);

        let held = carried_at(&lamps, eye, -Vec3::Z, Vec3::X, &[]);

        assert_eq!(held.len(), 1, "one in hand");
        assert!(
            (held[0].z - -HELD_OUT).abs() < 1e-5,
            "held at {}",
            held[0].z
        );
        assert!((held[0].y - (1.5 - HELD_DOWN)).abs() < 1e-5);
    }

    #[test]
    fn a_standing_lamp_lights_its_cell() {
        let maze = maze();
        let mut lamps = Lamps::new();
        let cell = maze::index(5, 5);
        lamps.put_down(cell);

        let here = cell_centre(cell);
        let lit = all(&maze, &lamps, Vec3::ZERO, -Vec3::Z, Vec3::X, &[])
            .into_iter()
            .filter(|light| light.casts)
            .any(|light| (light.position - here).length() < 1.0);

        assert!(lit, "nothing casts where the lamp was put down");
    }

    #[test]
    fn a_carried_lamp_follows_you() {
        // it moves with you, step for step, rather than being near you
        let maze = maze();
        let lamps = Lamps::new();
        let step = vec3(3.0, 0.0, -2.0);

        let here = all(&maze, &lamps, Vec3::ZERO, -Vec3::Z, Vec3::X, &[]);
        let there = all(&maze, &lamps, step, -Vec3::Z, Vec3::X, &[]);

        let carried = |lights: Vec<PointLight>| -> Vec<Vec3> {
            lights
                .into_iter()
                .filter(|l| l.casts)
                .map(|l| l.position)
                .collect()
        };

        for (before, after) in carried(here).into_iter().zip(carried(there)) {
            assert!(
                (after - before - step).length() < 1e-4,
                "it moved {:?} when you moved {:?}",
                after - before,
                step
            );
        }
    }

    #[test]
    fn only_the_two_cast() {
        let maze = maze();
        let mut lamps = Lamps::new();
        lamps.put_down(maze::index(2, 2));

        let casting = all(&maze, &lamps, Vec3::ZERO, -Vec3::Z, Vec3::X, &[])
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
                all(&maze, &lamps, Vec3::ZERO, -Vec3::Z, Vec3::X, &[]).len() <= MAX_POINT_LIGHTS,
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
    fn nothing_stands_where_you_start_or_finish() {
        for seed in 0..20 {
            let maze = Maze::carve(&mut StdRng::seed_from_u64(seed));
            let standing = fixed_cells(&maze);

            assert!(!standing.contains(&maze.start), "a brazier at the start");
            assert!(!standing.contains(&maze.exit), "a brazier at the exit");
        }
    }

    #[test]
    fn a_fixed_light_cannot_reach_past_a_wall() {
        // it cannot cast, so a wall does not stop it. Its range has to.
        let reach = crate::walls::CELL * 0.5 + crate::walls::THICK;

        assert!(
            FIXED_RANGE < reach,
            "at {} it reaches {} past the far face of its own walls",
            FIXED_RANGE,
            FIXED_RANGE - reach
        );
    }

    #[test]
    fn no_fixed_light_shines_through_stone() {
        // the same thing measured rather than argued: nothing a fixed light
        // touches is on the far side of a wall from it
        use blitzkit::collision::Ray;

        let maze = maze();
        let walls = crate::walls::colliders(&maze);
        let mut through = 0;

        for light in fixed(&maze) {
            for cell in 0..maze::CELLS {
                for height in [0.0f32, 1.0, crate::walls::TALL] {
                    let at = cell_centre(cell) + Vec3::Y * height;
                    let gap = at - light.position;
                    if gap.length() > light.range {
                        continue;
                    }
                    let blocked = walls.iter().any(|wall| {
                        Ray::new(light.position, gap)
                            .hit_aabb(wall)
                            .is_some_and(|hit| hit.distance < gap.length())
                    });
                    if blocked {
                        through += 1;
                    }
                }
            }
        }

        assert_eq!(through, 0, "{} lit points are behind a wall", through);
    }

    #[test]
    fn a_fixed_light_still_lights_its_own_floor() {
        // a range short enough to stay in the cell is no use if it stops short
        // of the floor it is standing on
        let maze = maze();

        for (light, cell) in fixed(&maze).into_iter().zip(fixed_cells(&maze)) {
            let floor = cell_centre(cell);

            assert!(
                (light.position - floor).length() < light.range,
                "its own floor is {} away and it reaches {}",
                (light.position - floor).length(),
                light.range
            );
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
