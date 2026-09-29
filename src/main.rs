//! A dark maze and two lamps to light it with. See `specs/`.
//!
//! Nothing draws yet. What this prints is the rules working: a carved maze,
//! how far the exit is, and the lamps being put down and taken up.

mod lamps;
mod maze;

use lamps::Lamps;
use maze::{Maze, HEIGHT, WIDTH};

fn main() {
    let mut rng = rand::thread_rng();
    let maze = Maze::carve(&mut rng);
    let steps = maze.distances_from(maze.start)[maze.exit].expect("there is a way out");

    println!(
        "a {} by {} maze, {} walls down for {} cells, {} dead ends left, \
         the exit {} steps from the door",
        WIDTH,
        HEIGHT,
        maze.openings(),
        maze::CELLS,
        maze.dead_ends().len(),
        steps
    );

    let mut lamps = Lamps::new();
    lamps.put_down(maze.start);
    println!(
        "\none lamp left at the door, {} still in hand",
        lamps.carried()
    );
    println!("lit cells: {:?}", lamps.standing());

    lamps.take_up(maze.start);
    println!(
        "taken up again, {} in hand and {:?} lit",
        lamps.carried(),
        lamps.standing()
    );
}
