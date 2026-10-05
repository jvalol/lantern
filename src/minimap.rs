//! A map of what your own light has fallen on, and nothing else. See
//! `specs/0006-what-the-light-touched.md`.
//!
//! Not a map of the maze: a record of where a candle of yours has been. The two
//! candles were already a decision about where to see, and this makes them a
//! decision about what to keep.

use blitzkit::geometry::quad::Quad;
use glam::{vec2, vec4, Vec2, Vec4};

use crate::lights::LAMP_RANGE;
use crate::maze::{self, beside, Maze, Side};
use crate::walls::CELL;

/// How many cells a candle records around itself, through open sides.
///
/// Its reach on the map comes from its reach in the world. Steps rather than
/// distance through the air: light turns a corner along a corridor and does not
/// go through a wall, and the maze already knows which is which.
pub const REACH: usize = (LAMP_RANGE / CELL) as usize;

/// How big a cell is drawn, and the line between two of them.
pub const CELL_PIXELS: f32 = 26.0;
pub const WALL_PIXELS: f32 = 3.0;

/// How far in from the corner it sits.
pub const MARGIN: f32 = 20.0;

pub const FLOOR: Vec4 = vec4(0.76, 0.80, 0.88, 0.92);
/// The card the map is drawn on, and how far it reaches past the cells.
///
/// Three lit cells on their own have no extent and no boundary, and read as a
/// mark on the wall rather than a map. The card gives the map its shape without
/// drawing a cell you have not lit.
///
/// Dark rather than grey. A pale card over a lit wall is the wall, and the
/// point is that this is over the world rather than in it.
pub const CARD: Vec4 = vec4(0.05, 0.055, 0.075, 0.62);
pub const CARD_PAD: f32 = 10.0;

/// The cells you have not lit yet, drawn on the card.
///
/// The card alone gives the map its shape. This says which of it is maze and
/// which is margin, and leaves the unexplored part something that can differ
/// cell by cell later rather than one flat tone.
pub const UNLIT: Vec4 = vec4(0.55, 0.58, 0.66, 0.14);
pub const WALL: Vec4 = vec4(0.10, 0.11, 0.14, 0.85);
/// You, in the one colour nothing else on the map uses.
///
/// Warm against warm was the first trouble: a candle mark and a player mark in
/// the same amber read as two candles. White fixed that and made another, which
/// is that the floor is nearly white itself and a white mark on it is a mark
/// you have to look for.
///
/// Blue rather than red. Red is the stronger contrast against a pale floor, but
/// it is a neighbour of the candles' amber at four pixels across, and telling
/// yourself from a candle is the whole job of this mark. Nothing else here is
/// blue, and it is dark enough to read against the floor as well.
pub const YOU: Vec4 = vec4(0.05, 0.16, 0.92, 1.0);

/// How much of a cell you take up, and how far the point reaches past you.
const YOU_WIDE: f32 = 0.45;
const POINT_FROM: f32 = 0.5;
const POINT_TO: f32 = 1.2;
const POINT_MARKS: usize = 5;
pub const CANDLE: Vec4 = vec4(1.0, 0.74, 0.35, 0.9);

/// What your light has fallen on. One flag per cell, and it only ever goes on.
#[derive(Debug, Clone)]
pub struct Map {
    written: Vec<bool>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Self {
        Self {
            written: vec![false; maze::CELLS],
        }
    }

    pub fn is_written(&self, cell: usize) -> bool {
        self.written.get(cell).copied().unwrap_or(false)
    }

    /// Writes down everything a candle standing in each of these cells reaches.
    ///
    /// Only ever adds. Walking away does not forget, and picking a candle back
    /// up does not unwrite what it lit.
    pub fn record(&mut self, maze: &Maze, candles: &[usize]) {
        for candle in candles {
            for cell in reaches(maze, *candle) {
                self.written[cell] = true;
            }
        }
    }
}

/// Every cell a candle in this one lights, itself included.
pub fn reaches(maze: &Maze, from: usize) -> Vec<usize> {
    let mut found = vec![from];
    let mut edge = std::collections::VecDeque::from([(from, 0usize)]);

    while let Some((here, steps)) = edge.pop_front() {
        if steps == REACH {
            continue;
        }

        for side in Side::ALL {
            if !maze.is_open(here, side) {
                continue;
            }
            let Some(next) = beside(here, side) else {
                continue;
            };
            if found.contains(&next) {
                continue;
            }

            found.push(next);
            edge.push_back((next, steps + 1));
        }
    }

    found
}

/// Where the top left of the map sits, given the window.
///
/// The bottom right. The text runs across the top for the full width of the
/// window, not down one side of it, so there is no corner up there to share
/// with it. A window too small for the map keeps it on screen rather than off
/// the edge of it.
pub fn corner(window: Vec2) -> Vec2 {
    let across = maze::WIDTH as f32 * CELL_PIXELS;
    let down = maze::HEIGHT as f32 * CELL_PIXELS;

    vec2(
        (window.x - across - MARGIN - CARD_PAD).max(MARGIN + CARD_PAD),
        (window.y - down - MARGIN - CARD_PAD).max(MARGIN + CARD_PAD),
    )
}

/// The whole map as quads: the floor you have lit, the walls between the bits
/// of it you have lit, the candles you have set down, and you.
///
/// Nothing for the way out. This is memory, not foresight.
pub fn quads(
    map: &Map,
    maze: &Maze,
    window: Vec2,
    you: usize,
    facing: Vec2,
    standing: &[usize],
) -> Vec<Quad> {
    let at = corner(window);
    let place = |cell: usize| {
        let (x, y) = maze::at(cell);

        at + vec2(x as f32, y as f32) * CELL_PIXELS
    };

    let mut quads = Vec::new();

    // the card, which is the whole maze's worth of room whether or not any of
    // it is lit yet
    let across = maze::WIDTH as f32 * CELL_PIXELS;
    let down = maze::HEIGHT as f32 * CELL_PIXELS;
    // a quad's position is its middle, and a cell's is too, so the card's
    // middle is half a maze along from the first cell rather than the corner
    let middle = at
        + vec2(
            (maze::WIDTH - 1) as f32 * CELL_PIXELS * 0.5,
            (maze::HEIGHT - 1) as f32 * CELL_PIXELS * 0.5,
        );
    quads.push(Quad::colored(
        middle,
        vec2(across + CARD_PAD * 2.0, down + CARD_PAD * 2.0),
        CARD,
    ));

    for cell in 0..maze::CELLS {
        quads.push(Quad::colored(place(cell), Vec2::splat(CELL_PIXELS), UNLIT));
    }

    for cell in 0..maze::CELLS {
        if !map.is_written(cell) {
            continue;
        }

        quads.push(Quad::colored(place(cell), Vec2::splat(CELL_PIXELS), FLOOR));
    }

    // a wall is only drawn where you have lit both sides of it, or the map
    // would say something about a cell you have never seen
    for cell in 0..maze::CELLS {
        if !map.is_written(cell) {
            continue;
        }

        for side in [Side::North, Side::West] {
            if maze.is_open(cell, side) {
                continue;
            }
            let Some(other) = beside(cell, side) else {
                continue;
            };
            if !map.is_written(other) {
                continue;
            }

            // a quad sits on its middle, and so does a cell, so a wall drawn
            // at the cell's own place runs through the middle of it: a cell
            // walled north and west came out as a cross. It belongs on the
            // edge the two cells share, half a cell out.
            let half = CELL_PIXELS * 0.5;
            let (size, from_middle) = match side {
                Side::North => (vec2(CELL_PIXELS, WALL_PIXELS), vec2(0.0, -half)),
                _ => (vec2(WALL_PIXELS, CELL_PIXELS), vec2(-half, 0.0)),
            };

            quads.push(Quad::colored(place(cell) + from_middle, size, WALL));
        }
    }

    for candle in standing {
        quads.push(Quad::colored(place(*candle), Vec2::splat(4.0), CANDLE));
    }

    // you, and where you are looking. A dot and one nub read as two specks and
    // said nothing about which way round they went. A dot with a tapering point
    // off it reads as an arrow whichever way it lies, and quads cannot turn.
    let middle = place(you);
    let wide = CELL_PIXELS * YOU_WIDE;
    quads.push(Quad::colored(middle, Vec2::splat(wide), YOU));

    let point = facing.normalize_or_zero();
    for mark in 0..POINT_MARKS {
        let along = mark as f32 / (POINT_MARKS - 1).max(1) as f32;
        let out = POINT_FROM + along * (POINT_TO - POINT_FROM);
        let size = wide * (1.0 - along * 0.7);

        quads.push(Quad::colored(
            middle + point * CELL_PIXELS * out,
            Vec2::splat(size),
            YOU,
        ));
    }

    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn maze() -> Maze {
        Maze::carve(&mut StdRng::seed_from_u64(3))
    }

    fn written(map: &Map) -> usize {
        (0..maze::CELLS).filter(|c| map.is_written(*c)).count()
    }

    #[test]
    fn a_lit_cell_is_recorded() {
        let maze = maze();
        let mut map = Map::new();
        let candle = maze::index(5, 5);

        map.record(&maze, &[candle]);

        assert!(map.is_written(candle));
    }

    #[test]
    fn the_dark_is_not_recorded() {
        let maze = maze();
        let mut map = Map::new();

        map.record(&maze, &[maze::index(0, 0)]);

        let untouched = (0..maze::CELLS).filter(|c| !map.is_written(*c)).count();
        assert!(untouched > 0, "one candle lit the whole maze");
        assert_eq!(written(&map), reaches(&maze, maze::index(0, 0)).len());
    }

    #[test]
    fn a_wall_stops_it_recording() {
        // a neighbour with a wall between is not reached in one step, whatever
        // the distance through the air says
        let maze = maze();
        let cell = (0..maze::CELLS)
            .find(|c| !maze.is_open(*c, Side::East) && beside(*c, Side::East).is_some())
            .expect("some cell is walled to the east");
        let walled_off = beside(cell, Side::East).expect("checked");

        let reached = reaches(&maze, cell);
        let steps = maze.distances_from(cell)[walled_off];

        if steps.map(|s| s > REACH).unwrap_or(true) {
            assert!(!reached.contains(&walled_off), "light went through a wall");
        }
    }

    #[test]
    fn it_reaches_as_far_as_a_lamp_does() {
        // the map's reach comes from the world's
        assert_eq!(REACH, (LAMP_RANGE / CELL) as usize);

        let maze = maze();
        let from = maze::index(8, 8);
        let steps = maze.distances_from(from);

        for cell in reaches(&maze, from) {
            assert!(
                steps[cell].map(|s| s <= REACH).unwrap_or(false),
                "cell {} is {:?} steps off and the reach is {}",
                cell,
                steps[cell],
                REACH
            );
        }
    }

    #[test]
    fn a_candle_left_behind_keeps_recording() {
        // it lights its corridor whether you are in it or not
        let maze = maze();
        let mut map = Map::new();
        let left = maze::index(2, 9);

        map.record(&maze, &[left]);
        let after_setting_down = written(&map);

        // you walk off; it is still there, so it is still recording
        map.record(&maze, &[left]);

        assert_eq!(written(&map), after_setting_down);
        assert!(after_setting_down > 1, "it lit only its own cell");
    }

    #[test]
    fn somebody_elses_light_records_nothing() {
        // the braziers were burning before you arrived. Only what is handed to
        // `record` writes, and the game hands it candles.
        let maze = maze();
        let mut map = Map::new();

        map.record(&maze, &[]);

        assert_eq!(written(&map), 0);
    }

    #[test]
    fn what_is_written_stays_written() {
        let maze = maze();
        let mut map = Map::new();
        map.record(&maze, &[maze::index(1, 1)]);
        let seen = written(&map);

        map.record(&maze, &[maze::index(14, 14)]);

        assert!(written(&map) > seen, "the second candle wrote nothing");
        assert!(map.is_written(maze::index(1, 1)), "the first was forgotten");
    }

    #[test]
    fn taking_a_candle_back_does_not_unwrite_it() {
        let maze = maze();
        let mut map = Map::new();
        let put_down = maze::index(7, 3);

        map.record(&maze, &[put_down]);
        let seen = written(&map);

        // picked up: nothing is handed to record any more
        map.record(&maze, &[]);

        assert_eq!(written(&map), seen);
    }

    #[test]
    fn the_way_out_is_not_on_the_map() {
        let maze = maze();
        let mut map = Map::new();
        map.record(&maze, &[maze.start]);

        let drawn = quads(&map, &maze, vec2(800.0, 600.0), maze.start, Vec2::Y, &[]);
        let exit = corner(vec2(800.0, 600.0))
            + vec2(maze::at(maze.exit).0 as f32, maze::at(maze.exit).1 as f32) * CELL_PIXELS;

        if !map.is_written(maze.exit) {
            // the unlit field covers every cell, the exit's included, which
            // says nothing about it. What must not be there is anything that
            // tells that cell apart from the rest of the dark.
            assert!(
                !drawn
                    .iter()
                    .any(|q| (q.position - exit).length() < 1.0 && q.color != UNLIT),
                "the way out is marked and nothing has lit it"
            );
        }
    }

    #[test]
    fn only_what_is_recorded_is_drawn() {
        let maze = maze();
        let mut map = Map::new();
        map.record(&maze, &[maze::index(4, 4)]);

        let floors = quads(
            &map,
            &maze,
            vec2(800.0, 600.0),
            maze::index(4, 4),
            Vec2::Y,
            &[],
        )
        .into_iter()
        .filter(|q| q.color == FLOOR)
        .count();

        assert_eq!(floors, written(&map));
    }

    #[test]
    fn a_wall_needs_both_sides_recorded() {
        // a wall drawn with one side dark would say something about a cell you
        // have never seen
        let maze = maze();
        let mut map = Map::new();
        map.record(&maze, &[maze::index(8, 8)]);

        let at = corner(vec2(800.0, 600.0));
        for quad in quads(
            &map,
            &maze,
            vec2(800.0, 600.0),
            maze::index(8, 8),
            Vec2::Y,
            &[],
        ) {
            if quad.color != WALL {
                continue;
            }

            let on = ((quad.position - at) / CELL_PIXELS).round();
            let cell = maze::index(on.x as usize, on.y as usize);

            assert!(map.is_written(cell), "a wall on an unlit cell");
        }
    }

    #[test]
    fn you_are_where_you_are() {
        let maze = maze();
        let mut map = Map::new();
        let you = maze::index(11, 6);
        map.record(&maze, &[you]);

        let window = vec2(800.0, 600.0);
        let mine = corner(window) + vec2(11.0, 6.0) * CELL_PIXELS;

        // a quad sits on its middle, so the mark's own position is the cell's,
        // within a fraction of a cell rather than within a whole one
        let found = quads(&map, &maze, window, you, Vec2::Y, &[])
            .into_iter()
            .filter(|q| q.color == YOU)
            .any(|q| (q.position - mine).length() < CELL_PIXELS * 0.1);

        assert!(found, "no mark where you are standing");
    }

    #[test]
    fn the_way_you_face_is_drawn_along_your_facing() {
        // a dot alone says where you are and nothing about which way round you
        // are, so the marks past it have to lie along where you look
        let maze = maze();
        let mut map = Map::new();
        let you = maze::index(8, 8);
        map.record(&maze, &[you]);

        let window = vec2(800.0, 600.0);
        let middle = corner(window) + vec2(8.0, 8.0) * CELL_PIXELS;
        let facing = vec2(0.0, -1.0);

        let marks: Vec<Vec2> = quads(&map, &maze, window, you, facing, &[])
            .into_iter()
            .filter(|q| q.color == YOU)
            .map(|q| q.position - middle)
            .filter(|off| off.length() > 1.0)
            .collect();

        assert!(marks.len() >= 3, "only {} marks past the dot", marks.len());

        for off in &marks {
            let along = off.normalize_or_zero().dot(facing);
            assert!(
                along > 0.95,
                "a mark lies {:?}, not along {:?}",
                off,
                facing
            );
        }
    }

    #[test]
    fn a_wall_lies_on_the_edge_between_two_cells() {
        // it lay on the cell's own middle, so a cell walled north and west
        // came out as a cross through it. Jake was looking at a dead end and
        // the map showed a plus sign.
        let maze = maze();
        let mut map = Map::new();
        for cell in 0..maze::CELLS {
            map.record(&maze, &[cell]);
        }

        let window = vec2(800.0, 600.0);
        let at = corner(window);
        let place = |cell: usize| {
            let (x, y) = maze::at(cell);
            at + vec2(x as f32, y as f32) * CELL_PIXELS
        };

        let walls: Vec<Vec2> = quads(&map, &maze, window, maze.start, Vec2::Y, &[])
            .into_iter()
            .filter(|q| q.color == WALL)
            .map(|q| q.position)
            .collect();

        assert!(!walls.is_empty(), "a carved maze has walls in it");

        let half = CELL_PIXELS * 0.5;
        for wall in &walls {
            let on_an_edge = (0..maze::CELLS).any(|cell| {
                let middle = place(cell);

                (*wall - (middle - vec2(0.0, half))).length() < 1e-3
                    || (*wall - (middle - vec2(half, 0.0))).length() < 1e-3
            });

            assert!(on_an_edge, "a wall at {:?} is on no cell's edge", wall);

            let through_a_cell = (0..maze::CELLS).any(|cell| (*wall - place(cell)).length() < 1e-3);

            assert!(!through_a_cell, "a wall at {:?} runs through a cell", wall);
        }
    }

    #[test]
    fn you_do_not_look_like_a_candle() {
        assert_ne!(YOU, CANDLE, "you and your candles are the same colour");
    }

    #[test]
    fn you_stand_out_against_the_floor() {
        // perceived brightness, not a channel: the floor is pale and a mark of
        // its own brightness is a mark you have to hunt for
        let bright = |c: Vec4| 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;

        let gap = bright(FLOOR) - bright(YOU);
        assert!(gap > 0.5, "you are only {} off the floor", gap);
    }

    #[test]
    fn it_keeps_out_of_the_way_of_the_text() {
        // the text runs across the whole top, so the map lives at the bottom
        let at = corner(vec2(1600.0, 900.0));
        let across = maze::WIDTH as f32 * CELL_PIXELS;
        let down = maze::HEIGHT as f32 * CELL_PIXELS;

        assert!(at.y > 900.0 * 0.5, "it is up in the text at {}", at.y);
        assert!(at.x > 1600.0 * 0.5, "it is on the left at {}", at.x);
        assert!(
            at.x + across <= 1600.0 - MARGIN + 1e-3,
            "it runs off the side"
        );
        assert!(
            at.y + down <= 900.0 - MARGIN + 1e-3,
            "it runs off the bottom"
        );

        // and a window too small for it stays on screen rather than off it
        let cramped = corner(vec2(100.0, 100.0));
        assert!(cramped.x >= MARGIN && cramped.y >= MARGIN, "{:?}", cramped);
    }
}
