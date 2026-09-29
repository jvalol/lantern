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

use crate::lamps::Lamps;
use crate::lights;
use crate::maze::{self, Maze};
use crate::player::Player;
use crate::walls::{self, cell_centre};

const WALL_COLOR: glam::Vec4 = vec4(0.62, 0.58, 0.52, 1.0);

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
    player: Player,
    lamps: Lamps,

    held: [bool; 4],
    locked: bool,
    wants_lock: bool,
    quitting: bool,
    readout: RenderText,
}

impl LanternGame {
    pub fn new() -> Self {
        let maze = Maze::carve(&mut StdRng::from_entropy());
        let walls = walls::colliders(&maze);
        let mut player = Player::at(cell_centre(maze.start));
        player.yaw = facing_a_way_out(&maze);

        Self {
            maze,
            walls,
            mesh: None,
            player,
            lamps: Lamps::new(),
            held: [false; 4],
            locked: false,
            wants_lock: true,
            quitting: false,
            readout: RenderText {
                position: vec2(20.0, 20.0),
                color: vec4(1.0, 1.0, 1.0, 0.85),
                size: 20.0,
                ..Default::default()
            },
        }
    }

    /// Which cell you are standing in.
    fn cell(&self) -> usize {
        (0..maze::CELLS)
            .min_by(|a, b| {
                let d = |cell: &usize| (cell_centre(*cell) - self.player.position).length();
                d(a).total_cmp(&d(b))
            })
            .expect("the maze has cells")
    }

    fn wish(&self) -> Vec3 {
        let [ahead, back, left, right] = self.held;
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
        _window_size: (f32, f32),
    ) {
    }

    fn update(
        &mut self,
        dt: f32,
        geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
    ) {
        geometry.reset();
        text_renderer.reset();

        self.player.walk(self.wish(), dt, &self.walls);

        let steps = self.maze.distances_from(self.cell())[self.maze.exit];
        self.readout.text = if self.cell() == self.maze.exit {
            String::from("out")
        } else {
            format!(
                "{} in hand, {} down   {} cells from the way out   space to put down, e to take up",
                self.lamps.carried(),
                self.lamps.standing().len(),
                steps.unwrap_or(0)
            )
        };
        text_renderer.render_texts.push(self.readout.clone());
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        let Some(mesh) = self.mesh else {
            return;
        };

        // no sun. The maze is lit by the lamps and the dim fixed ones, which is
        // the whole point of it being dark.
        scene.light.intensity = 0.0;
        scene.light.ambient = Vec3::splat(0.03);

        scene.push_colored(mesh, &Transform::default(), WALL_COLOR);

        // held out in front and below the eye, not at it
        let hand = self.player.eye() + self.player.forward() * lights::HELD_OUT
            - Vec3::Y * lights::HELD_DOWN;
        for light in lights::all(&self.maze, &self.lamps, hand, self.player.right()) {
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
            KeyboardKey::A | KeyboardKey::Left => self.held[2] = down,
            KeyboardKey::D | KeyboardKey::Right => self.held[3] = down,
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
            KeyboardKey::Space => {
                self.lamps.put_down(self.cell());
            }
            KeyboardKey::E => {
                self.lamps.take_up(self.cell());
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
            self.held = [false; 4];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_lamp_is_put_down_where_you_stand() {
        let mut game = LanternGame::new();
        let here = game.cell();

        game.lamps.put_down(here);

        assert_eq!(game.lamps.standing(), vec![here]);
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
        game.held = [true; 4];

        game.focus_changed(false);

        assert_eq!(game.held, [false; 4], "a held key does not stay held");
    }
}
