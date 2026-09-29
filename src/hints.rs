//! Asking the way out, and deciding how much to be told. See
//! `specs/0003-hints.md`.

use blitzkit::mesh::MeshData;
use glam::vec3;

use crate::maze::{self, Maze, Side};

/// How much you want to be told. One mechanism at four strengths, so turning
/// it up is a decision about how much rather than about what kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Hint {
    #[default]
    Off,
    Whisper,
    Trail,
    Whole,
}

/// How far a trail runs. Five is a corner and then some: far enough to commit
/// to a direction, short enough that you are still choosing.
const TRAIL: usize = 5;

impl Hint {
    pub const ALL: [Hint; 4] = [Hint::Off, Hint::Whisper, Hint::Trail, Hint::Whole];

    /// The next setting up, and round to off again. Defined off `ALL` so the
    /// order the settings come in is written down once.
    pub fn next(self) -> Self {
        let at = Self::ALL.iter().position(|hint| *hint == self).unwrap_or(0);

        Self::ALL[(at + 1) % Self::ALL.len()]
    }

    /// How many cells of the route it shows.
    pub fn cells(self) -> usize {
        match self {
            Hint::Off => 0,
            Hint::Whisper => 1,
            Hint::Trail => TRAIL,
            Hint::Whole => usize::MAX,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Hint::Off => "off",
            Hint::Whisper => "a hint",
            Hint::Trail => "hintier",
            Hint::Whole => "hintiest",
        }
    }

    /// How brightly, before the ambient term it is multiplied against.
    pub fn glow(self) -> f32 {
        match self {
            Hint::Off => 0.0,
            Hint::Whisper => 0.16,
            Hint::Trail => 0.34,
            Hint::Whole => 0.55,
        }
    }
}

/// How far above the floor a mark sits, and how wide it is.
const MARK_LIFT: f32 = 0.03;
const MARK_WIDE: f32 = 0.9;
const MARK_THIN: f32 = 0.05;

/// A mark on the floor: a flat lozenge, and closed, so it is lit rather than
/// looked through.
pub fn mark() -> MeshData {
    crate::candle::stretched(
        MeshData::sphere(18, 6),
        vec3(MARK_WIDE, MARK_THIN, MARK_WIDE),
    )
}

/// Where one sits in a cell.
pub fn mark_at(cell: usize) -> glam::Vec3 {
    crate::walls::cell_centre(cell) + glam::Vec3::Y * MARK_LIFT
}

/// How far every cell is from the way out.
///
/// Computed once, because the maze does not change. The route from anywhere is
/// the walk down it, which is why going the wrong way leads you out from where
/// you end up rather than back to where you were.
pub struct Route {
    to_the_way_out: Vec<Option<usize>>,
}

impl Route {
    pub fn of(maze: &Maze) -> Self {
        Self {
            to_the_way_out: maze.distances_from(maze.exit),
        }
    }

    /// The whole way out from a cell, the cell itself not included.
    pub fn from(&self, maze: &Maze, cell: usize) -> Vec<usize> {
        let mut route = Vec::new();
        let mut here = cell;

        while let Some(distance) = self.to_the_way_out[here] {
            if distance == 0 {
                break;
            }

            // the neighbour one step nearer the way out, through a side that
            // is actually open
            let next = Side::ALL.iter().filter_map(|side| {
                if !maze.is_open(here, *side) {
                    return None;
                }
                let beside = maze::beside(here, *side)?;
                (self.to_the_way_out[beside] == Some(distance - 1)).then_some(beside)
            });

            match next.into_iter().next() {
                Some(step) => {
                    route.push(step);
                    here = step;
                }
                None => break,
            }
        }

        route
    }

    /// The part of it this setting shows.
    pub fn shown(&self, maze: &Maze, cell: usize, hint: Hint) -> Vec<usize> {
        let mut route = self.from(maze, cell);
        route.truncate(hint.cells());
        route
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

    #[test]
    fn the_route_arrives() {
        for seed in 0..20 {
            let maze = maze(seed);
            let route = Route::of(&maze).from(&maze, maze.start);

            assert_eq!(
                route.last(),
                Some(&maze.exit),
                "seed {} stops somewhere else",
                seed
            );
        }
    }

    #[test]
    fn every_step_goes_through_an_opening() {
        for seed in 0..20 {
            let maze = maze(seed);
            let route = Route::of(&maze).from(&maze, maze.start);
            let mut here = maze.start;

            for step in route {
                let open = Side::ALL.iter().any(|side| {
                    maze.is_open(here, *side) && maze::beside(here, *side) == Some(step)
                });

                assert!(
                    open,
                    "seed {}: {} to {} is through a wall",
                    seed, here, step
                );
                here = step;
            }
        }
    }

    #[test]
    fn it_is_as_short_as_the_maze_allows() {
        for seed in 0..20 {
            let maze = maze(seed);
            let shortest = maze.distances_from(maze.start)[maze.exit].expect("there is a way out");

            assert_eq!(
                Route::of(&maze).from(&maze, maze.start).len(),
                shortest,
                "seed {} takes the scenic route",
                seed
            );
        }
    }

    #[test]
    fn at_the_way_out_there_is_nothing_left() {
        let maze = maze(3);

        assert!(Route::of(&maze).from(&maze, maze.exit).is_empty());
    }

    #[test]
    fn off_shows_nothing() {
        let maze = maze(3);

        assert!(Route::of(&maze)
            .shown(&maze, maze.start, Hint::Off)
            .is_empty());
    }

    #[test]
    fn each_setting_shows_more_than_the_last() {
        let maze = maze(3);
        let route = Route::of(&maze);
        let seen = |hint| route.shown(&maze, maze.start, hint).len();

        for pair in Hint::ALL.windows(2) {
            assert!(
                seen(pair[1]) > seen(pair[0]),
                "{:?} shows {} and {:?} shows {}",
                pair[0],
                seen(pair[0]),
                pair[1],
                seen(pair[1])
            );
        }
    }

    #[test]
    fn a_whisper_is_one_and_a_trail_is_five() {
        let maze = maze(3);
        let route = Route::of(&maze);

        assert_eq!(route.shown(&maze, maze.start, Hint::Whisper).len(), 1);
        assert_eq!(route.shown(&maze, maze.start, Hint::Trail).len(), TRAIL);
    }

    #[test]
    fn the_whole_way_is_the_whole_way() {
        let maze = maze(3);
        let route = Route::of(&maze);

        assert_eq!(
            route.shown(&maze, maze.start, Hint::Whole),
            route.from(&maze, maze.start)
        );
    }

    #[test]
    fn it_leads_out_from_wherever_you_are() {
        // walk the wrong way and it leads out from there, not back to the start
        let maze = maze(3);
        let route = Route::of(&maze);
        let wrong_way = *route
            .from(&maze, maze.start)
            .first()
            .expect("the start is not the way out");

        for cell in 0..maze::CELLS {
            if maze.distances_from(maze.exit)[cell].is_none() {
                continue;
            }
            let from_here = route.from(&maze, cell);

            if cell != maze.exit {
                assert_eq!(from_here.last(), Some(&maze.exit), "cell {}", cell);
            }
        }

        assert_ne!(
            route.from(&maze, wrong_way).first(),
            Some(&maze.start),
            "it sent us back where we came from"
        );
    }

    #[test]
    fn it_cycles_back_to_off() {
        let mut hint = Hint::Off;

        for _ in 0..Hint::ALL.len() {
            hint = hint.next();
        }

        assert_eq!(hint, Hint::Off);
    }
}
