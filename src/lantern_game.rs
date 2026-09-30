//! The maze, walked. See `specs/0002-walking-it.md`.

use blitzkit::camera::Camera;
use blitzkit::collision::Aabb;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::mesh::Transform;
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::Game;
use glam::{vec2, vec3, vec4, Vec2, Vec3};
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::hints::{Hint, Route};
use crate::lamps::Lamps;
use crate::lights;
use crate::maze::{self, Maze};
use crate::minimap::Map;
use crate::player::Player;
use crate::walls::{self, cell_centre};

const WALL_COLOR: glam::Vec4 = vec4(0.62, 0.58, 0.52, 1.0);
/// Wax, lit by its own flame from above like anything else.
const WAX_LOOK: glam::Vec4 = vec4(0.92, 0.88, 0.76, 1.0);

/// The flame. The engine has no per-object emissive, but colour multiplies the
/// ambient term as well as the diffuse one, so a large colour against a dim
/// ambient is what makes it look lit from inside.
const FLAME_LOOK: glam::Vec4 = vec4(170.0, 130.0, 60.0, 1.0);

/// What a mark on the way out is tinted, before its setting brightens it. Pale
/// and cool, so it is not mistaken for something burning.
pub const MARK_TINT: Vec3 = vec3(0.72, 0.86, 0.95);

/// How much light there is with nothing lighting it.
///
/// This is the one number that decides whether a wall you have no light on is
/// visible. At 0.03 the whole maze was faintly there at once, walls and
/// distance no object, which looks like light bleeding through stone.
///
/// It cannot simply go to nothing, because a flame is drawn by multiplying it:
/// no ambient, no flame. Dropping it means multiplying the flames back up by
/// the same amount, which is what separates the two.
const AMBIENT: f32 = 0.006;

/// The hand holding a candle, warm and close to the flame lighting it.
const HAND_LOOK: glam::Vec4 = vec4(0.78, 0.56, 0.42, 1.0);

/// The braziers already burning in the maze: cold iron, and a cold flame, so
/// they read as someone else's light rather than a candle you dropped.
const BRAZIER_LOOK: glam::Vec4 = vec4(0.26, 0.28, 0.34, 1.0);
const BRAZIER_FLAME_LOOK: glam::Vec4 = vec4(40.0, 55.0, 100.0, 1.0);

/// Which way to look at the start: down a way out of the first cell, rather
/// than at whichever wall happens to be ahead.
fn facing_a_way_out(maze: &Maze) -> f32 {
    use crate::maze::Side;
    use std::f32::consts::{FRAC_PI_2, PI};

    Side::ALL
        .iter()
        .find(|side| maze.is_open(maze.start, **side))
        .map(|side| match side {
            Side::North => 0.0,
            Side::South => PI,
            Side::East => FRAC_PI_2,
            Side::West => -FRAC_PI_2,
        })
        .unwrap_or(0.0)
}

pub struct LanternGame {
    maze: Maze,
    walls: Vec<Aabb>,
    mesh: Option<MeshId>,
    /// The candle: wax and flame, so a light on the wall has something making
    /// it and the thing making it is lit.
    wax: Option<MeshId>,
    flame: Option<MeshId>,
    hand: Option<MeshId>,
    brazier: Option<MeshId>,
    brazier_flame: Option<MeshId>,
    mark: Option<MeshId>,
    threshold: Option<MeshId>,
    tile: Option<MeshId>,
    face: Option<MeshId>,
    faces: Vec<(Vec3, Vec3)>,
    field: Vec<(Vec3, f32)>,
    /// Whether you have stepped through the door. Once, and it stays.
    out: bool,
    height: f32,
    ending: RenderText,
    hint: Hint,
    /// What your own light has fallen on, and whether you are looking at it.
    map: Map,
    show_map: bool,
    route: Route,
    hint_line: RenderText,
    player: Player,
    lamps: Lamps,

    /// Forward, back, strafe left, strafe right, turn left, turn right.
    held: [bool; 6],
    /// What the readout wraps at, which is the window less a margin.
    width: f32,
    locked: bool,
    wants_lock: bool,
    quitting: bool,
    readout: RenderText,
    controls: RenderText,
}

impl LanternGame {
    pub fn new() -> Self {
        let maze = Maze::carve(&mut StdRng::from_entropy());
        // the braziers stand in the maze, so they stop you and stop a candle
        // the same way a wall does
        let route = Route::of(&maze);
        let field = crate::field::tiles(&maze);
        let faces = crate::field::outer_faces(&maze);
        let mut walls = walls::colliders(&maze);
        walls.extend(
            lights::fixed_cells(&maze)
                .into_iter()
                .map(|cell| crate::brazier::solid(cell_centre(cell))),
        );
        let mut player = Player::at(cell_centre(maze.start));
        player.yaw = facing_a_way_out(&maze);

        Self {
            maze,
            walls,
            mesh: None,
            wax: None,
            flame: None,
            hand: None,
            brazier: None,
            brazier_flame: None,
            mark: None,
            threshold: None,
            tile: None,
            face: None,
            faces,
            field,
            out: false,
            height: 0.0,
            ending: RenderText {
                color: vec4(1.0, 1.0, 1.0, 0.95),
                size: 32.0,
                centered: true,
                text: String::from("You made it!"),
                ..Default::default()
            },
            hint: Hint::default(),
            map: Map::new(),
            show_map: true,
            route,
            player,
            lamps: Lamps::new(),
            held: [false; 6],
            width: 800.0,
            locked: false,
            wants_lock: true,
            quitting: false,
            readout: RenderText {
                position: vec2(20.0, 20.0),
                color: vec4(1.0, 1.0, 1.0, 0.9),
                size: 20.0,
                ..Default::default()
            },
            hint_line: RenderText {
                position: vec2(20.0, 104.0),
                color: vec4(1.0, 1.0, 1.0, 0.55),
                size: 20.0,
                ..Default::default()
            },
            controls: RenderText {
                position: vec2(20.0, 52.0),
                color: vec4(1.0, 1.0, 1.0, 0.55),
                size: 20.0,
                text: String::from(
                    "Press the left and right arrow keys to rotate, wasd keys or the mouse to move, and press space to place or pick up a candle",
                ),
                ..Default::default()
            },
        }
    }

    /// Which cell you are standing in.
    /// Whether you have walked out through the door.
    ///
    /// Crossing the line the wall stood on, rather than arriving in the cell.
    /// Arriving happens a stride before the doorway and is over by the time you
    /// walk through it, which is the wrong moment for the thing to end on.
    fn beyond_the_door(&self) -> bool {
        let (dx, dy) = self.maze.way_out().step();
        let out = vec3(dx as f32, 0.0, dy as f32);
        let line = crate::field::doorstep(&self.maze);

        (self.player.position - line).dot(out) > 0.0
    }

    /// Every light the scene should carry. Out on the grass that is the
    /// braziers and nothing else: the candles are done.
    fn lit(&self) -> Vec<blitzkit::lighting::PointLight> {
        if self.out {
            return lights::fixed(&self.maze);
        }

        lights::all(
            &self.maze,
            &self.lamps,
            self.player.eye(),
            self.player.forward(),
            self.player.right(),
            &self.walls,
        )
    }

    /// How far there is left to go, and what you are carrying.
    ///
    /// Out on the grass this is not shown at all. The nearest cell to somewhere
    /// outside the maze is still a cell, so it went on counting at someone who
    /// had already left.
    fn update_readout(&mut self) {
        let steps = self.maze.distances_from(self.cell())[self.maze.exit].unwrap_or(0);

        self.readout.text = format!(
            "{} in hand, {} down   {} {} from the way out",
            self.lamps.carried(),
            self.lamps.standing().len(),
            steps,
            if steps == 1 { "cell" } else { "cells" }
        );
    }

    fn cell(&self) -> usize {
        (0..maze::CELLS)
            .min_by(|a, b| {
                let d = |cell: &usize| (cell_centre(*cell) - self.player.position).length();
                d(a).total_cmp(&d(b))
            })
            .expect("the maze has cells")
    }

    fn wish(&self) -> Vec3 {
        let [ahead, back, left, right, _, _] = self.held;
        let mut wish = Vec3::ZERO;

        if ahead {
            wish += self.player.forward();
        }
        if back {
            wish -= self.player.forward();
        }
        if right {
            wish += self.player.right();
        }
        if left {
            wish -= self.player.right();
        }

        wish
    }
}

impl Default for LanternGame {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for LanternGame {
    fn load(&mut self, renderer: &mut Renderer) {
        self.mesh = Some(renderer.add_mesh(&walls::mesh(&self.maze)));
        self.wax = Some(renderer.add_mesh(&crate::candle::wax()));
        self.flame = Some(renderer.add_mesh(&crate::candle::flame()));
        self.hand = Some(renderer.add_mesh(&crate::candle::hand()));
        self.brazier = Some(renderer.add_mesh(&crate::brazier::stand()));
        self.brazier_flame = Some(renderer.add_mesh(&crate::brazier::flame()));
        self.mark = Some(renderer.add_mesh(&crate::hints::mark()));
        self.threshold = Some(renderer.add_mesh(&crate::way_out::threshold()));
        self.tile = Some(renderer.add_mesh(&crate::field::tile()));
        self.face = Some(renderer.add_mesh(&blitzkit::mesh::MeshData::cube()));

        let reach = maze::WIDTH as f32 * walls::CELL;
        renderer.set_scene_bounds(Aabb::new(
            vec3(-reach, -walls::TALL, -reach),
            vec3(reach, walls::TALL * 2.0, reach),
        ));
    }

    fn before_frame(&mut self, renderer: &mut Renderer) {
        if self.wants_lock != self.locked {
            self.locked = renderer.set_cursor_locked(self.wants_lock) && self.wants_lock;
        }
    }

    fn initialize(
        &mut self,
        _geometry: &mut Geometry,
        _text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
        window_size: (f32, f32),
    ) {
        self.resized(window_size);
    }

    fn resized(&mut self, window_size: (f32, f32)) {
        self.width = window_size.0;
        self.height = window_size.1;
    }

    fn update(
        &mut self,
        dt: f32,
        geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        sound_system: &SoundSystem,
    ) {
        geometry.reset();
        text_renderer.reset();

        // the arrows turn you and the mouse looks. Strafing is A and D.
        let [_, _, _, _, turn_left, turn_right] = self.held;
        if turn_left {
            self.player.turn(-crate::player::TURN * dt);
        }
        if turn_right {
            self.player.turn(crate::player::TURN * dt);
        }

        self.player.walk(self.wish(), dt, &self.walls);

        // the moment of it, once. Walking further out does not do it again.
        if !self.out && self.beyond_the_door() {
            self.out = true;
            sound_system.queue(crate::chime::chime());
        }

        self.update_readout();

        // your own light, and only yours: the candles in hand light where you
        // are, and the ones you set down go on lighting where they are. See
        // spec 0006.
        let mut candles: Vec<usize> = self.lamps.standing();
        if self.lamps.carried() > 0 && !self.out {
            candles.push(self.cell());
        }
        self.map.record(&self.maze, &candles);

        if self.show_map && !self.out {
            let facing = self.player.forward();
            for quad in crate::minimap::quads(
                &self.map,
                &self.maze,
                vec2(self.width, self.height),
                self.cell(),
                vec2(facing.x, facing.z),
                &self.lamps.standing(),
            ) {
                geometry.push_quad(&quad);
            }
        }

        // two lines at heights of our own, because a wrapped line's leading is
        // the font's and it is too tight to read
        let wide = vec2(
            self.width - 40.0,
            blitzkit::renderer::render_text::UNBOUNDED_F32,
        );
        self.readout.bounds = wide;
        self.controls.bounds = wide;

        if !self.out {
            text_renderer.render_texts.push(self.readout.clone());
        }
        self.hint_line.bounds = wide;
        self.hint_line.text = format!("Hint: {}. Press h to make it hintier.", self.hint.name());
        text_renderer.render_texts.push(self.controls.clone());
        text_renderer.render_texts.push(self.hint_line.clone());

        if self.out {
            self.ending.position = vec2(self.width * 0.5, self.height * 0.5);
            text_renderer.render_texts.push(self.ending.clone());
        }
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        let Some(mesh) = self.mesh else {
            return;
        };

        // no sun. The maze is lit by the lamps and the dim fixed ones, which is
        // the whole point of it being dark.
        scene.light.intensity = 0.0;
        scene.light.ambient = Vec3::splat(AMBIENT);

        scene.push_colored(mesh, &Transform::default(), WALL_COLOR);

        // a candle, not a ball: the flame sits above the wax, so the wax is lit
        // by it rather than being the one unlit thing in the maze
        let carried = lights::carried_at(
            &self.lamps,
            self.player.eye(),
            self.player.forward(),
            self.player.right(),
            &self.walls,
        );

        // the grass outside, drawn whether you are out on it or not. Seeing it
        // through the doorway is most of what makes the door worth walking to.
        // the labyrinth from outside. Its braziers are shut inside it and your
        // candles are out by the time you can look back at it, so without a
        // face on it the thing you just walked out of is not there.
        if let Some(face) = self.face {
            let lit = crate::field::STONE_TINT * (crate::field::STONE_GLOW / AMBIENT);
            let look = vec4(lit.x, lit.y, lit.z, 1.0);

            for (at, size) in &self.faces {
                scene.push_colored(face, &Transform::at(*at).with_scale(*size), look);
            }
        }

        if let Some(tile) = self.tile {
            for (at, glow) in &self.field {
                let lit = crate::field::TINT * (glow / AMBIENT);
                scene.push_colored(tile, &Transform::at(*at), vec4(lit.x, lit.y, lit.z, 1.0));
            }
        }

        // light coming in at the door, which is what makes the way out visible
        // from down a corridor rather than only once you are standing in it
        if let Some(threshold) = self.threshold {
            let glow = crate::way_out::GLOW / AMBIENT;
            let tint = crate::way_out::TINT * glow;

            scene.push_colored(
                threshold,
                &Transform::at(crate::way_out::threshold_at(&self.maze)),
                vec4(tint.x, tint.y, tint.z, 1.0),
            );
        }

        // the way out, as far along it as the setting shows. A mark is
        // geometry, so a wall hides it: turning it up lights more of the route
        // rather than handing you a map.
        if let Some(mark) = self.mark {
            let glow = self.hint.glow() / AMBIENT;
            let look = vec4(
                MARK_TINT.x * glow,
                MARK_TINT.y * glow,
                MARK_TINT.z * glow,
                1.0,
            );

            for cell in self.route.marks(&self.maze, self.cell(), self.hint) {
                scene.push_colored(mark, &Transform::at(crate::hints::mark_at(cell)), look);
            }
        }

        // the fixed lights, with something making each of them. A light with
        // nothing at it reads as a smear rather than a lamp.
        if let (Some(stand), Some(lit)) = (self.brazier, self.brazier_flame) {
            for cell in lights::fixed_cells(&self.maze) {
                let foot = cell_centre(cell);
                scene.push_colored(stand, &Transform::at(foot), BRAZIER_LOOK);
                scene.push_colored(
                    lit,
                    &Transform::at(foot + Vec3::Y * crate::brazier::FLAME_AT),
                    BRAZIER_FLAME_LOOK,
                );
            }
        }

        // out on the grass the candles are done, which is the mechanic ending
        // rather than the screen saying so
        if let (Some(wax), Some(flame), Some(hand)) = (self.wax, self.flame, self.hand) {
            let carried: Vec<Vec3> = if self.out { Vec::new() } else { carried };
            let standing_cells: Vec<usize> = if self.out {
                Vec::new()
            } else {
                self.lamps.standing()
            };
            let standing = standing_cells
                .into_iter()
                .map(|cell| (cell_centre(cell) + Vec3::Y * lights::STANDS, false));
            let held = carried.iter().map(|at| (*at, true));

            for (flame_at, in_hand) in standing.chain(held) {
                let foot = flame_at - Vec3::Y * crate::candle::FLAME_HEIGHT;

                scene.push_colored(wax, &Transform::at(foot), WAX_LOOK);
                scene.push_colored(flame, &Transform::at(flame_at), FLAME_LOOK);

                // one you are holding has a hand round it, so the light in
                // front of you is yours rather than something floating there
                if in_hand {
                    let grip = foot + Vec3::Y * crate::candle::HAND_GRIP;
                    scene.push_colored(hand, &Transform::at(grip), HAND_LOOK);
                }
            }
        }

        for light in self.lit() {
            scene.push_light(light);
        }

        camera.position = self.player.eye();
        camera.target = self.player.eye() + self.player.facing();
        camera.fov_y = 75f32.to_radians();
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let down = input.state == KeyboardKeyState::Pressed;

        match input.key {
            KeyboardKey::W | KeyboardKey::Up => self.held[0] = down,
            KeyboardKey::S | KeyboardKey::Down => self.held[1] = down,
            KeyboardKey::A => self.held[2] = down,
            KeyboardKey::D => self.held[3] = down,
            KeyboardKey::Left => self.held[4] = down,
            KeyboardKey::Right => self.held[5] = down,
            _ => {}
        }

        if !down || input.repeat {
            return;
        }

        match input.key {
            // escape lets the cursor go first, and quits from there
            KeyboardKey::Escape => {
                if self.locked {
                    self.wants_lock = false;
                } else {
                    self.quitting = true;
                }
            }
            // one key: take up the one you are standing at, or put one down
            KeyboardKey::M => {
                if down {
                    self.show_map = !self.show_map;
                }
            }
            KeyboardKey::H => {
                if down {
                    self.hint = self.hint.next();
                }
            }
            KeyboardKey::Space => {
                let here = self.cell();
                if !self.lamps.take_up(here) {
                    self.lamps.put_down(here);
                }
            }
            _ => {}
        }
    }

    fn process_mouse(&mut self, input: blitzkit::mouse::MouseInput) {
        if input.is_pressed() && !self.locked {
            self.wants_lock = true;
        }
    }

    fn mouse_motion(&mut self, delta: Vec2) {
        if self.locked {
            self.player.look(delta);
        }
    }

    fn is_quitting(&self) -> bool {
        self.quitting
    }

    fn focus_changed(&mut self, focus: bool) {
        if !focus {
            self.wants_lock = false;
            self.held = [false; 6];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hint_key_cycles_back_to_off() {
        let mut game = LanternGame::new();
        let press = |key| KeyboardInput::new(key, KeyboardKeyState::Pressed, false);

        assert_eq!(game.hint, Hint::Off);

        let mut seen = vec![game.hint];
        for _ in 1..Hint::ALL.len() {
            game.process_keyboard(press(KeyboardKey::H));
            assert!(
                !seen.contains(&game.hint),
                "{:?} came round twice",
                game.hint
            );
            seen.push(game.hint);
        }

        game.process_keyboard(press(KeyboardKey::H));
        assert_eq!(game.hint, Hint::Off, "it never comes back to off");
    }

    #[test]
    fn no_marks_once_you_are_out() {
        // nothing left to point at, whatever the setting
        let mut game = LanternGame::new();
        game.player = crate::player::Player::at(cell_centre(game.maze.exit));

        for hint in Hint::ALL {
            game.hint = hint;

            assert!(
                game.route
                    .shown(&game.maze, game.cell(), game.hint)
                    .is_empty(),
                "{:?} still marks something at the way out",
                hint
            );
        }
    }

    #[test]
    fn it_ends_when_you_walk_through_not_when_you_arrive() {
        // arriving happens a stride before the doorway and is over by the time
        // you walk through it, which is the wrong moment to end on
        let mut game = LanternGame::new();
        let (dx, dy) = game.maze.way_out().step();
        let out = vec3(dx as f32, 0.0, dy as f32);

        game.player = crate::player::Player::at(cell_centre(game.maze.exit));
        assert!(
            !game.beyond_the_door(),
            "standing in the cell already counted as out"
        );

        game.player = crate::player::Player::at(cell_centre(game.maze.exit) + out * walls::CELL);
        assert!(game.beyond_the_door(), "a step outside did not count");
    }

    #[test]
    fn the_candles_go_out_when_you_do() {
        let mut game = LanternGame::new();

        assert!(
            game.lit().iter().any(|light| light.casts),
            "nothing was casting to begin with"
        );

        game.out = true;

        assert!(
            !game.lit().iter().any(|light| light.casts),
            "a candle is still burning out on the grass"
        );
        assert!(
            !game.lit().is_empty(),
            "the braziers went out too, and they are not yours to put out"
        );
    }

    #[test]
    fn the_ending_is_centred_in_the_window() {
        let mut game = LanternGame::new();
        game.resized((800.0, 600.0));

        assert!(game.ending.centered);

        game.out = true;
        game.ending.position = vec2(game.width * 0.5, game.height * 0.5);

        assert_eq!(game.ending.position, vec2(400.0, 300.0));
    }

    #[test]
    fn the_map_key_shows_and_hides_it() {
        let mut game = LanternGame::new();
        let press = |key| KeyboardInput::new(key, KeyboardKeyState::Pressed, false);

        assert!(game.show_map, "it starts shown");

        game.process_keyboard(press(KeyboardKey::M));
        assert!(!game.show_map);

        game.process_keyboard(press(KeyboardKey::M));
        assert!(game.show_map);
    }

    #[test]
    fn carrying_a_candle_writes_where_you_are() {
        let mut game = LanternGame::new();
        let here = game.cell();

        assert!(
            !game.map.is_written(here),
            "written before anything happened"
        );

        let standing = game.lamps.standing();
        let mut candles = standing.clone();
        candles.push(here);
        game.map.record(&game.maze, &candles);

        assert!(game.map.is_written(here));
    }

    #[test]
    fn one_cell_is_a_cell() {
        let mut game = LanternGame::new();
        let steps = game.maze.distances_from(game.maze.exit);
        let one_away = (0..maze::CELLS)
            .find(|cell| steps[*cell] == Some(1))
            .expect("something adjoins the way out");

        game.player = crate::player::Player::at(cell_centre(one_away));
        game.update_readout();

        assert!(
            game.readout.text.contains("1 cell from"),
            "{}",
            game.readout.text
        );
    }

    #[test]
    fn you_do_not_start_facing_a_wall() {
        use crate::maze::Side;

        for _ in 0..20 {
            let game = LanternGame::new();
            let yaw = game.player.yaw;
            let side = Side::ALL
                .iter()
                .copied()
                .find(|s| {
                    let want = match s {
                        Side::North => 0.0,
                        Side::South => std::f32::consts::PI,
                        Side::East => std::f32::consts::FRAC_PI_2,
                        Side::West => -std::f32::consts::FRAC_PI_2,
                    };
                    (yaw - want).abs() < 1e-4
                })
                .expect("the yaw is one of the four");

            assert!(
                game.maze.is_open(game.maze.start, side),
                "started looking at a wall"
            );
        }
    }

    #[test]
    fn you_start_at_the_door() {
        let game = LanternGame::new();

        assert_eq!(game.cell(), game.maze.start);
    }

    #[test]
    fn space_puts_one_down_and_takes_it_back() {
        let mut game = LanternGame::new();
        let here = game.cell();
        let press = |key| KeyboardInput::new(key, KeyboardKeyState::Pressed, false);

        game.process_keyboard(press(KeyboardKey::Space));
        assert_eq!(game.lamps.standing(), vec![here], "it went down");

        game.process_keyboard(press(KeyboardKey::Space));
        assert!(game.lamps.standing().is_empty(), "and came back up");
        assert_eq!(game.lamps.carried(), crate::lamps::LAMPS);
    }

    #[test]
    fn space_takes_up_before_it_puts_down() {
        // standing at a candle with one still in hand, you want the one at your
        // feet rather than a second one beside it
        let mut game = LanternGame::new();
        let here = game.cell();
        game.lamps.put_down(here);
        let press = |key| KeyboardInput::new(key, KeyboardKeyState::Pressed, false);

        game.process_keyboard(press(KeyboardKey::Space));

        assert!(game.lamps.standing().is_empty());
    }

    #[test]
    fn escape_lets_the_cursor_go_before_it_quits() {
        let mut game = LanternGame::new();
        game.locked = true;
        let press = |key| KeyboardInput::new(key, KeyboardKeyState::Pressed, false);

        game.process_keyboard(press(KeyboardKey::Escape));
        assert!(!game.wants_lock, "it let the cursor go");
        assert!(!game.is_quitting(), "and did not quit");

        game.locked = false;
        game.process_keyboard(press(KeyboardKey::Escape));
        assert!(game.is_quitting(), "and quits from there");
    }

    #[test]
    fn losing_focus_stops_you_walking() {
        let mut game = LanternGame::new();
        game.held = [true; 6];

        game.focus_changed(false);

        assert_eq!(game.held, [false; 6], "a held key does not stay held");
    }
}
