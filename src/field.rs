//! The grass outside the door. See `specs/0005-getting-out.md`.
//!
//! Every light is spoken for, six braziers and two candles against spec 0020's
//! eight, so the field is not lit: it is coloured as an unclamped multiplier
//! the way the flames are, brightest at the door and falling away into the
//! night. What looks like moonlight is the tiles knowing how far out they are.

use blitzkit::mesh::MeshData;
use glam::{vec3, Vec3};

use crate::maze::{self, beside, Maze};
use crate::walls::{cell_centre, wall_box, CELL, THICK};

/// How big a patch of grass is. A cell, so the ground outside is on the same
/// grid as the ground in.
pub const TILE: f32 = CELL;

/// How far it runs: to each side of the door, and away from it.
const ACROSS: i32 = 11;
const OUT: i32 = 13;

/// How far out the light gets before there is none of it.
const REACH: f32 = 34.0;

pub const TINT: Vec3 = vec3(0.30, 0.47, 0.24);
pub const GLOW: f32 = 0.34;

/// The outside of the labyrinth, which nothing lights either. The braziers are
/// shut inside it and the candles are out by the time you can see it, so
/// without this the thing you just walked out of is not there at all.
pub const STONE_TINT: Vec3 = vec3(0.56, 0.58, 0.62);
pub const STONE_GLOW: f32 = 0.15;

/// How thick a face is. A skin laid against the outer wall rather than the
/// wall itself, so it never lights anything on the inside.
const SKIN: f32 = 0.06;

/// One patch: flat, its top flush with the floor inside.
pub fn tile() -> MeshData {
    crate::candle::stretched(MeshData::cube(), vec3(TILE, THICK, TILE))
}

/// Where the doorway opens onto it.
pub fn doorstep(maze: &Maze) -> Vec3 {
    let (dx, dy) = maze.way_out().step();
    let out = vec3(dx as f32, 0.0, dy as f32);

    cell_centre(maze.exit) + out * (CELL * 0.5 + THICK * 0.5)
}

/// Every patch of it, and how brightly each one glows.
///
/// Worked out once, because neither the maze nor the field moves.
pub fn tiles(maze: &Maze) -> Vec<(Vec3, f32)> {
    let (dx, dy) = maze.way_out().step();
    let out = vec3(dx as f32, 0.0, dy as f32);
    let across = out.cross(Vec3::Y);
    let door = doorstep(maze);

    let mut patches = Vec::new();

    for away in 0..OUT {
        for beside in -ACROSS..=ACROSS {
            let at = door + out * (TILE * (away as f32 + 0.5)) + across * (TILE * beside as f32)
                - Vec3::Y * THICK * 0.5;

            let gone = ((at - door).length() / REACH).clamp(0.0, 1.0);
            patches.push((at, GLOW * (1.0 - gone) * (1.0 - gone)));
        }
    }

    patches
}

/// The outer walls seen from the field: one face per outward side, and none
/// across the doorway.
pub fn outer_faces(maze: &Maze) -> Vec<(Vec3, Vec3)> {
    let mut faces = Vec::new();

    for cell in 0..maze::CELLS {
        for side in crate::maze::Side::ALL {
            if beside(cell, side).is_some() {
                continue;
            }
            if cell == maze.exit && side == maze.way_out() {
                continue;
            }

            let (dx, dy) = side.step();
            let out = vec3(dx as f32, 0.0, dy as f32);
            let wall = wall_box(cell, side);
            let mut size = wall.size();

            if dx != 0 {
                size.x = SKIN;
            } else {
                size.z = SKIN;
            }

            faces.push((wall.center() + out * (THICK * 0.5 + SKIN), size));
        }
    }

    faces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn maze(seed: u64) -> Maze {
        Maze::carve(&mut StdRng::seed_from_u64(seed))
    }

    #[test]
    fn the_field_is_outside_the_maze() {
        // sharing the floor's plane with the floor would fight it for every
        // pixel, so none of it may lie over the maze
        for seed in 0..10 {
            let maze = maze(seed);
            let half = maze::WIDTH as f32 * CELL * 0.5;

            for (at, _) in tiles(&maze) {
                let inside = at.x.abs() < half - 1e-3 && at.z.abs() < half - 1e-3;

                assert!(
                    !inside,
                    "seed {}: a patch at {:?} is over the maze",
                    seed, at
                );
            }
        }
    }

    #[test]
    fn it_is_brightest_at_the_door() {
        let maze = maze(3);
        let door = doorstep(&maze);
        let patches = tiles(&maze);

        let nearest = patches
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("there is a field");

        assert!(
            (nearest.0 - door).length() < TILE * 1.5,
            "the brightest patch is {} from the door",
            (nearest.0 - door).length()
        );
    }

    #[test]
    fn it_fades_into_the_night() {
        let maze = maze(3);
        let door = doorstep(&maze);
        let furthest = tiles(&maze)
            .into_iter()
            .max_by(|a, b| {
                let d = |p: &(Vec3, f32)| (p.0 - door).length();
                d(a).total_cmp(&d(b))
            })
            .expect("there is a field");

        assert!(
            furthest.1 < GLOW * 0.1,
            "the far edge still glows at {} of {}",
            furthest.1,
            GLOW
        );
    }

    #[test]
    fn the_outside_is_faced_everywhere_but_the_doorway() {
        for seed in 0..10 {
            let maze = maze(seed);
            let outward = (0..maze::CELLS)
                .flat_map(|cell| crate::maze::Side::ALL.map(move |side| (cell, side)))
                .filter(|(cell, side)| beside(*cell, *side).is_none())
                .count();

            assert_eq!(
                outer_faces(&maze).len(),
                outward - 1,
                "seed {}: every outward side but the door",
                seed
            );
        }
    }

    #[test]
    fn nothing_stands_across_the_doorway() {
        for seed in 0..10 {
            let maze = maze(seed);
            let door = wall_box(maze.exit, maze.way_out()).center();

            for (at, _) in outer_faces(&maze) {
                assert!(
                    (at - door).length() > THICK,
                    "seed {}: a face is across the doorway",
                    seed
                );
            }
        }
    }

    #[test]
    fn the_faces_lie_outside_the_walls() {
        // laid against the outer wall, not in it: a face inside would fight the
        // wall for pixels, and one further out would float
        let maze = maze(3);

        for (at, _) in outer_faces(&maze) {
            let wall = (0..maze::CELLS)
                .flat_map(|cell| crate::maze::Side::ALL.map(move |side| (cell, side)))
                .filter(|(cell, side)| beside(*cell, *side).is_none())
                .map(|(cell, side)| wall_box(cell, side).center())
                .min_by(|a, b| (*a - at).length().total_cmp(&(*b - at).length()))
                .expect("there are outer walls");

            let apart = (at - wall).length();

            assert!(
                apart > THICK * 0.5 && apart < THICK,
                "a face sits {} from its wall",
                apart
            );
        }
    }

    #[test]
    fn it_is_flush_with_the_floor_inside() {
        let top = tile().bounds().max.y + tiles(&maze(3))[0].0.y;

        assert!(top.abs() < 1e-4, "the grass is {} above the floor", top);
    }
}
