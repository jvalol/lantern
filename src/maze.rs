//! The maze: a grid of cells with walls between them, and exactly one route
//! from anywhere to anywhere. See `specs/0001-the-two-lamps.md`.
//!
//! Carved by walking from a cell to an unvisited neighbour and backing up when
//! there is none, which is a spanning tree: every cell reached, one route
//! between any two. Then half the dead ends are opened out, because a maze that
//! is all dead ends is all backtracking.

use rand::Rng;

pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 16;
pub const CELLS: usize = WIDTH * HEIGHT;

/// How many dead ends get a second way out. A perfect maze is nothing but dead
/// ends and the walk back from them; a fully braided one has no dead ends and
/// no tension. Half.
pub const BRAID: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    North,
    South,
    East,
    West,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::North, Side::South, Side::East, Side::West];

    pub fn step(self) -> (i32, i32) {
        match self {
            Side::North => (0, -1),
            Side::South => (0, 1),
            Side::East => (1, 0),
            Side::West => (-1, 0),
        }
    }

    fn opposite(self) -> Side {
        match self {
            Side::North => Side::South,
            Side::South => Side::North,
            Side::East => Side::West,
            Side::West => Side::East,
        }
    }

    fn bit(self) -> u8 {
        match self {
            Side::North => 1,
            Side::South => 2,
            Side::East => 4,
            Side::West => 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Maze {
    /// Which sides of each cell are open. A wall stands where a side is shut.
    open: Vec<u8>,
    pub start: usize,
    pub exit: usize,
}

pub fn index(x: usize, y: usize) -> usize {
    y * WIDTH + x
}

pub fn at(cell: usize) -> (usize, usize) {
    (cell % WIDTH, cell / WIDTH)
}

/// The cell on the given side of this one, if it is on the board.
pub fn beside(cell: usize, side: Side) -> Option<usize> {
    let (x, y) = at(cell);
    let (dx, dy) = side.step();
    let (nx, ny) = (x as i32 + dx, y as i32 + dy);

    ((0..WIDTH as i32).contains(&nx) && (0..HEIGHT as i32).contains(&ny))
        .then(|| index(nx as usize, ny as usize))
}

impl Maze {
    /// Carves a maze. The way out is the furthest cell that has an outer wall
    /// to cut, so there is always an outside for it to lead to.
    pub fn carve(rng: &mut impl Rng) -> Self {
        let mut maze = Self {
            open: vec![0; CELLS],
            start: 0,
            exit: 0,
        };

        maze.carve_only(rng);
        maze.braid(rng);
        maze.exit = maze.furthest_on_the_edge();
        maze.open[maze.exit] |= maze.way_out().bit();
        maze
    }

    /// The spanning tree: every cell reached, one route between any two.
    fn carve_only(&mut self, rng: &mut impl Rng) {
        let mut seen = vec![false; CELLS];
        let mut route = vec![self.start];
        seen[self.start] = true;

        while let Some(cell) = route.last().copied() {
            let mut ways: Vec<Side> = Side::ALL
                .iter()
                .copied()
                .filter(|side| beside(cell, *side).is_some_and(|next| !seen[next]))
                .collect();

            if ways.is_empty() {
                route.pop();
                continue;
            }

            let side = ways.remove(rng.gen_range(0..ways.len()));
            let next = beside(cell, side).expect("it was just checked");

            self.open[cell] |= side.bit();
            self.open[next] |= side.opposite().bit();
            seen[next] = true;
            route.push(next);
        }
    }

    /// Opens a second way out of some of the dead ends, which makes loops.
    fn braid(&mut self, rng: &mut impl Rng) {
        let mut ends = self.dead_ends();
        let opening = (ends.len() as f32 * BRAID) as usize;

        for _ in 0..opening {
            if ends.is_empty() {
                break;
            }
            let cell = ends.remove(rng.gen_range(0..ends.len()));

            // any wall with a cell behind it will do, and the shut ones are
            // what is left after the one way in
            let shut: Vec<Side> = Side::ALL
                .iter()
                .copied()
                .filter(|side| !self.is_open(cell, *side) && beside(cell, *side).is_some())
                .collect();

            if let Some(side) = shut.get(rng.gen_range(0..shut.len().max(1))) {
                let next = beside(cell, *side).expect("it was just checked");
                self.open[cell] |= side.bit();
                self.open[next] |= side.opposite().bit();
            }
        }
    }

    /// Cells with exactly one way in and out.
    pub fn dead_ends(&self) -> Vec<usize> {
        (0..CELLS)
            .filter(|cell| self.open[*cell].count_ones() == 1)
            .collect()
    }

    pub fn is_open(&self, cell: usize, side: Side) -> bool {
        self.open[cell] & side.bit() != 0
    }

    /// How many steps every cell is from this one.
    ///
    /// Breadth first, which in a maze with loops in it is the difference
    /// between the shortest way and the first way. Taking cells off the end
    /// instead walks depth first and records whatever route it wandered in on:
    /// on seed 0 that was 148 steps where the maze allows 84.
    pub fn distances_from(&self, cell: usize) -> Vec<Option<usize>> {
        let mut steps = vec![None; CELLS];
        steps[cell] = Some(0);
        let mut edge = std::collections::VecDeque::from([cell]);

        while let Some(here) = edge.pop_front() {
            let so_far = steps[here].expect("it is on the edge because it was reached");

            for side in Side::ALL {
                if !self.is_open(here, side) {
                    continue;
                }
                let Some(next) = beside(here, side) else {
                    continue;
                };
                if steps[next].is_none() {
                    steps[next] = Some(so_far + 1);
                    edge.push_back(next);
                }
            }
        }

        steps
    }

    /// Which way the door out of the exit faces.
    ///
    /// A cell on the edge has at least one side with nothing beyond it. A
    /// corner has two, and either will do.
    pub fn way_out(&self) -> Side {
        Side::ALL
            .iter()
            .copied()
            .find(|side| beside(self.exit, *side).is_none())
            .expect("the way out is on the edge, so it has an outward side")
    }

    /// The furthest cell from the start that has an outer wall to cut.
    ///
    /// Furthest of all is usually somewhere in the middle, and a cell in the
    /// middle has nothing to be a door in. Picking from the edge brings it
    /// nearer; that is the price of a way out you can see.
    fn furthest_on_the_edge(&self) -> usize {
        let steps = self.distances_from(self.start);

        (0..CELLS)
            .filter(|cell| Side::ALL.iter().any(|side| beside(*cell, *side).is_none()))
            .filter_map(|cell| steps[cell].map(|steps| (steps, cell)))
            .max()
            .map(|(_, cell)| cell)
            .expect("the edge is reachable")
    }
}

#[cfg(test)]
mod way_out_tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn maze(seed: u64) -> Maze {
        Maze::carve(&mut StdRng::seed_from_u64(seed))
    }

    fn on_the_edge(cell: usize) -> bool {
        let (x, y) = at(cell);

        x == 0 || y == 0 || x == WIDTH - 1 || y == HEIGHT - 1
    }

    #[test]
    fn the_way_out_is_on_the_edge() {
        for seed in 0..20 {
            let maze = maze(seed);

            assert!(
                on_the_edge(maze.exit),
                "seed {}: the way out is at {:?}, in the middle",
                seed,
                at(maze.exit)
            );
        }
    }

    #[test]
    fn the_door_is_open() {
        for seed in 0..20 {
            let maze = maze(seed);

            assert!(
                maze.is_open(maze.exit, maze.way_out()),
                "seed {}: the door is walled up",
                seed
            );
        }
    }

    #[test]
    fn the_door_faces_outwards() {
        for seed in 0..20 {
            let maze = maze(seed);

            assert!(
                beside(maze.exit, maze.way_out()).is_none(),
                "seed {}: the door opens into the maze",
                seed
            );
        }
    }

    #[test]
    fn there_is_still_a_way_there() {
        for seed in 0..20 {
            let maze = maze(seed);

            assert!(
                maze.distances_from(maze.start)[maze.exit].is_some(),
                "seed {}: no way out at all",
                seed
            );
        }
    }

    #[test]
    fn it_is_still_a_long_way_off() {
        // picking from the edge brings it nearer, and it has to stay a walk
        let least = (0..20)
            .map(|seed| {
                let maze = maze(seed);
                maze.distances_from(maze.start)[maze.exit].expect("reachable")
            })
            .min()
            .expect("some mazes");

        assert!(
            least >= 20,
            "the nearest way out over 20 mazes is {}",
            least
        );
    }
}

#[cfg(test)]
mod distance_tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn maze(seed: u64) -> Maze {
        Maze::carve(&mut StdRng::seed_from_u64(seed))
    }

    #[test]
    fn a_step_changes_the_distance_by_one_at_most() {
        // the property depth first does not have: two cells with a door
        // between them cannot be five steps apart
        for seed in 0..20 {
            let maze = maze(seed);
            let steps = maze.distances_from(maze.exit);

            for cell in 0..CELLS {
                for side in Side::ALL {
                    if !maze.is_open(cell, side) {
                        continue;
                    }
                    let Some(next) = beside(cell, side) else {
                        continue;
                    };
                    let (Some(here), Some(there)) = (steps[cell], steps[next]) else {
                        continue;
                    };

                    assert!(
                        here.abs_diff(there) <= 1,
                        "seed {}: {} and {} have a door between them and are {} and {}",
                        seed,
                        cell,
                        next,
                        here,
                        there
                    );
                }
            }
        }
    }

    #[test]
    fn it_is_the_same_distance_back() {
        for seed in 0..20 {
            let maze = maze(seed);

            assert_eq!(
                maze.distances_from(maze.start)[maze.exit],
                maze.distances_from(maze.exit)[maze.start],
                "seed {} is further one way than the other",
                seed
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn maze(seed: u64) -> Maze {
        Maze::carve(&mut StdRng::seed_from_u64(seed))
    }

    /// How many walls were knocked down. A maze with one route between any two
    /// cells has exactly one fewer than it has cells; more means a loop.
    fn openings(maze: &Maze) -> usize {
        maze.open
            .iter()
            .map(|sides| sides.count_ones() as usize)
            .sum::<usize>()
            / 2
    }

    #[test]
    fn every_maze_can_be_finished() {
        for seed in 0..40 {
            let maze = maze(seed);

            assert!(
                maze.distances_from(maze.start)[maze.exit].is_some(),
                "seed {} has no way out",
                seed
            );
        }
    }

    #[test]
    fn every_cell_can_be_reached() {
        for seed in 0..40 {
            let maze = maze(seed);
            let reached = maze
                .distances_from(maze.start)
                .into_iter()
                .filter(|steps| steps.is_some())
                .count();

            assert_eq!(reached, CELLS, "seed {} walls a cell off", seed);
        }
    }

    #[test]
    fn there_are_loops() {
        // a tree has one fewer opening than it has cells. Anything more is a
        // second route to somewhere, which is the point of braiding.
        for seed in 0..40 {
            let maze = maze(seed);

            assert!(
                openings(&maze) > CELLS - 1,
                "seed {} is still a perfect maze",
                seed
            );
        }
    }

    #[test]
    fn braiding_takes_out_about_half_the_dead_ends() {
        for seed in 0..20 {
            let mut perfect = Maze {
                open: vec![0; CELLS],
                start: 0,
                exit: 0,
            };
            perfect.carve_only(&mut StdRng::seed_from_u64(seed));
            let before = perfect.dead_ends().len();

            let after = maze(seed).dead_ends().len();

            assert!(
                after < before,
                "seed {}: {} dead ends became {}",
                seed,
                before,
                after
            );
        }
    }

    #[test]
    fn the_exit_is_not_on_the_doorstep() {
        for seed in 0..40 {
            let maze = maze(seed);
            let steps = maze.distances_from(maze.start)[maze.exit].expect("reachable");

            assert!(
                steps > WIDTH,
                "seed {}: the exit is {} steps away",
                seed,
                steps
            );
        }
    }

    #[test]
    fn a_wall_is_solid() {
        // a side that is shut has no way through it, and the cell on the other
        // side agrees
        for seed in 0..20 {
            let maze = maze(seed);

            for cell in 0..CELLS {
                for side in Side::ALL {
                    let Some(next) = beside(cell, side) else {
                        // one side of one cell opens off the board on purpose,
                        // and that is the door out. Nothing else does.
                        let door = cell == maze.exit && side == maze.way_out();
                        assert!(
                            door || !maze.is_open(cell, side),
                            "seed {}: an edge cell opens off the board and is not the way out",
                            seed
                        );
                        continue;
                    };

                    assert_eq!(
                        maze.is_open(cell, side),
                        maze.is_open(next, side.opposite()),
                        "seed {}: cells {} and {} disagree about the wall between them",
                        seed,
                        cell,
                        next
                    );
                }
            }
        }
    }
}
