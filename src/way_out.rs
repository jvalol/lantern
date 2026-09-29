//! The door on the edge of the maze, and the light coming in. See
//! `specs/0004-the-way-out.md`.

use blitzkit::mesh::MeshData;
use glam::{vec3, Vec3};

use crate::maze::Maze;
use crate::walls::{cell_centre, CELL};

/// How far across the cell the patch of light lies, and how thick.
const WIDE: f32 = 2.0;
const THIN: f32 = 0.05;
const LIFT: f32 = 0.025;

/// How far from the cell's middle towards the door it sits. It is light coming
/// in through the opening, so it belongs at the opening.
const TOWARDS_THE_DOOR: f32 = CELL * 0.34;

/// How brightly it glows, before the ambient term it multiplies. Above the
/// strongest hint, so the way out is never the second brightest thing on the
/// floor.
pub const GLOW: f32 = 0.78;

/// Pale and cold, which a hint is not.
pub const TINT: Vec3 = vec3(0.86, 0.90, 1.0);

/// The patch itself: flat, and closed, so it is lit rather than looked through.
pub fn threshold() -> MeshData {
    crate::candle::stretched(MeshData::sphere(20, 6), vec3(WIDE, THIN, WIDE))
}

/// Where it lies: in the doorway rather than in the middle of the cell.
pub fn threshold_at(maze: &Maze) -> Vec3 {
    let (dx, dy) = maze.way_out().step();
    let towards = vec3(dx as f32, 0.0, dy as f32) * TOWARDS_THE_DOOR;

    cell_centre(maze.exit) + towards + Vec3::Y * LIFT
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn maze(seed: u64) -> Maze {
        Maze::carve(&mut StdRng::seed_from_u64(seed))
    }

    #[test]
    fn the_threshold_lies_in_the_doorway() {
        for seed in 0..20 {
            let maze = maze(seed);
            let middle = cell_centre(maze.exit);
            let patch = threshold_at(&maze);

            let towards_the_door = (patch - middle).length();

            assert!(
                towards_the_door > CELL * 0.25,
                "seed {}: it sits {} from the middle, which is the middle",
                seed,
                towards_the_door
            );
            assert!(
                towards_the_door < CELL * 0.5,
                "seed {}: it sits {} out, which is inside the wall",
                seed,
                towards_the_door
            );
        }
    }

    #[test]
    fn it_is_as_wide_as_the_door() {
        let across = threshold().bounds().size();

        assert!(
            across.x > CELL * 0.6 && across.z > CELL * 0.6,
            "{:?} does not fill a doorway {} across",
            across,
            CELL
        );
    }

    #[test]
    fn it_does_not_read_as_a_hint() {
        use crate::hints::Hint;

        assert!(
            GLOW > Hint::Whole.glow(),
            "the way out glows {} and the strongest hint {}",
            GLOW,
            Hint::Whole.glow()
        );
        assert!(
            (TINT - crate::lantern_game::MARK_TINT).length() > 0.1,
            "it is the same colour as a hint"
        );
    }
}
