//! The maze as geometry: where the walls stand, and the boxes that stop you
//! walking through them. See `specs/0002-walking-it.md`.

use blitzkit::collision::Aabb;
use blitzkit::mesh::MeshData;
use glam::{vec3, Vec3};

use crate::maze::{self, Maze, Side};

/// How wide a cell is, and the unit everything here is measured in.
pub const CELL: f32 = 3.0;

/// How thick a wall is, and how tall.
pub const THICK: f32 = 0.3;
pub const TALL: f32 = 2.6;

/// The middle of a cell's floor, in the world.
pub fn cell_centre(cell: usize) -> Vec3 {
    let (x, y) = maze::at(cell);
    let offset = |n: usize, of: usize| (n as f32 - (of as f32 - 1.0) * 0.5) * CELL;

    vec3(offset(x, maze::WIDTH), 0.0, offset(y, maze::HEIGHT))
}

/// The box a wall on this side of this cell occupies.
pub fn wall_box(cell: usize, side: Side) -> Aabb {
    let centre = cell_centre(cell);
    let half = CELL * 0.5;

    let (offset, size) = match side {
        Side::North => (vec3(0.0, 0.0, -half), vec3(CELL + THICK, TALL, THICK)),
        Side::South => (vec3(0.0, 0.0, half), vec3(CELL + THICK, TALL, THICK)),
        Side::West => (vec3(-half, 0.0, 0.0), vec3(THICK, TALL, CELL + THICK)),
        Side::East => (vec3(half, 0.0, 0.0), vec3(THICK, TALL, CELL + THICK)),
    };

    Aabb::from_center_size(centre + offset + Vec3::Y * TALL * 0.5, size)
}

/// Every wall still standing, as a box. A side that is open has none.
///
/// Each wall is between two cells and would otherwise be counted twice, so only
/// the north and west sides are walked, plus the south and east edges of the
/// grid where there is no neighbour to have counted it.
pub fn colliders(maze: &Maze) -> Vec<Aabb> {
    let mut boxes = Vec::new();

    for cell in 0..maze::CELLS {
        for side in [Side::North, Side::West, Side::South, Side::East] {
            if maze.is_open(cell, side) {
                continue;
            }
            let shared = matches!(side, Side::South | Side::East);
            if shared && maze::beside(cell, side).is_some() {
                continue;
            }
            boxes.push(wall_box(cell, side));
        }
    }

    boxes
}

/// The slab over a cell, at the top of the walls.
///
/// A cell wide, so neighbouring slabs meet edge to edge over the middle of
/// whatever wall is between them and leave no seam to see the dark through.
pub fn ceiling_box(cell: usize) -> Aabb {
    Aabb::from_center_size(
        cell_centre(cell) + Vec3::Y * (TALL + THICK * 0.5),
        vec3(CELL, THICK, CELL),
    )
}

/// The maze as one mesh: a floor under every cell, a slab over it, and a slab
/// at every wall.
///
/// The ceiling was out of scope until a player looked up and found the tops of
/// the walls, lit, with nothing above them. A maze you are meant to be lost in
/// is a maze you cannot see out of.
pub fn mesh(maze: &Maze) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let mut add = |bounds: &Aabb| {
        let cube = MeshData::cube();
        let first = vertices.len() as u32;
        let (centre, size) = (bounds.center(), bounds.size());

        for vertex in &cube.vertices {
            let at = Vec3::from(vertex.position) * size + centre;
            vertices.push(blitzkit::mesh::Vertex::new(
                at.to_array(),
                vertex.normal,
                vertex.uv,
            ));
        }
        indices.extend(cube.indices.iter().map(|index| index + first));
    };

    for cell in 0..maze::CELLS {
        let floor = Aabb::from_center_size(
            cell_centre(cell) - Vec3::Y * THICK * 0.5,
            vec3(CELL, THICK, CELL),
        );
        add(&floor);
        add(&ceiling_box(cell));
    }

    for bounds in colliders(maze) {
        add(&bounds);
    }

    MeshData::new(vertices, indices)
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
    fn every_wall_is_solid() {
        // as many boxes as there are walls, counting each shared one once
        for seed in 0..10 {
            let maze = maze(seed);
            let mut walls = 0;

            for cell in 0..maze::CELLS {
                for side in Side::ALL {
                    if maze.is_open(cell, side) {
                        continue;
                    }
                    let shared = matches!(side, Side::South | Side::East);
                    if shared && maze::beside(cell, side).is_some() {
                        continue;
                    }
                    walls += 1;
                }
            }

            assert_eq!(colliders(&maze).len(), walls, "seed {}", seed);
        }
    }

    #[test]
    fn an_open_side_is_walkable() {
        let maze = maze(1);
        let cell = (0..maze::CELLS)
            .find(|cell| maze.is_open(*cell, Side::East))
            .expect("some cell opens east");
        let gap = wall_box(cell, Side::East);

        assert!(
            !colliders(&maze).iter().any(|b| b.center() == gap.center()),
            "an open side got a collider"
        );
    }

    #[test]
    fn the_walls_are_where_the_maze_put_them() {
        // a wall on a cell's east side is the same wall as its neighbour's west
        let cell = maze::index(3, 4);
        let east = wall_box(cell, Side::East);
        let neighbour = maze::beside(cell, Side::East).expect("not the edge");
        let west = wall_box(neighbour, Side::West);

        assert!(
            (east.center() - west.center()).length() < 1e-4,
            "{:?} against {:?}",
            east.center(),
            west.center()
        );
    }

    #[test]
    fn a_cell_is_a_cell_wide() {
        let apart = cell_centre(maze::index(1, 0)) - cell_centre(maze::index(0, 0));

        assert!((apart.length() - CELL).abs() < 1e-5, "{}", apart.length());
    }

    #[test]
    fn the_doorway_has_no_wall() {
        // the way out is a hole in the outer wall, so nothing is built there
        for seed in 0..20 {
            let maze = maze(seed);
            let door = wall_box(maze.exit, maze.way_out());

            assert!(
                !colliders(&maze)
                    .iter()
                    .any(|wall| (wall.center() - door.center()).length() < 1e-4),
                "seed {}: the doorway is walled up",
                seed
            );
        }
    }

    #[test]
    fn the_maze_is_centred() {
        let first = cell_centre(0);
        let last = cell_centre(maze::CELLS - 1);

        assert!((first + last).length() < 1e-4, "the grid is off centre");
    }

    #[test]
    fn a_ceiling_sits_on_top_of_the_walls() {
        let bounds = ceiling_box(0);

        assert!((bounds.min.y - TALL).abs() < 1e-5);
        assert!((bounds.max.y - (TALL + THICK)).abs() < 1e-5);
    }

    #[test]
    fn the_ceiling_leaves_no_gap_between_cells() {
        // cell 0 and the cell east of it, whose slabs must meet
        let east = maze::beside(0, Side::East).expect("the grid is wider than one cell");
        let (near, far) = (ceiling_box(0), ceiling_box(east));

        assert!(far.min.x <= near.max.x + 1e-5);
        assert!(far.min.x >= near.max.x - 1e-5);
    }
}
