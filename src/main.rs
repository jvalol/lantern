//! A dark maze and two lamps to light it with. See `specs/`.

mod brazier;
mod candle;
mod lamps;
mod lantern_game;
mod lights;
mod maze;
mod player;
mod walls;

use blitzkit::start;
use lantern_game::LanternGame;

fn main() {
    start("lantern", Box::new(LanternGame::new()));
}
