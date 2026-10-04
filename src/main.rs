//! A dark maze and two lamps to light it with. See `specs/`.

mod brazier;
mod candle;
mod chime;
mod field;
mod hints;
mod lamps;
mod lantern_game;
mod lights;
mod maze;
mod minimap;
mod player;
mod walls;
mod way_out;

use blitzkit::start;
use lantern_game::LanternGame;

/// Whether this run is only here to be photographed, for `refresh-screenshots`
/// in the project above.
pub fn staged() -> bool {
    std::env::args().any(|arg| arg == "--screenshot")
}

fn main() {
    start("lantern", Box::new(LanternGame::new()));
}
