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

fn main() {
    start("lantern", Box::new(LanternGame::new()));
}
