//! Standing in the maze, looking around, and walking into things. See
//! `specs/0002-walking-it.md`.

use blitzkit::collision::{move_and_slide, Aabb, Sphere};
use glam::{vec3, Vec3};

/// How wide you are. Narrow enough to pass a corridor, wide enough not to
/// squeeze through a corner.
pub const RADIUS: f32 = 0.45;

/// Eye height off the floor.
pub const EYE: f32 = 1.5;

pub const SPEED: f32 = 4.2;
pub const LOOK: f32 = 0.0022;

/// How far up and down you can look, short of straight up, where a camera's up
/// vector has nothing to be square to.
const PITCH_LIMIT: f32 = 1.45;

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Player {
    pub fn at(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    /// Which way you are looking.
    pub fn facing(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();

        vec3(sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch)
    }

    /// Which way is forward on the floor: facing, flattened.
    pub fn forward(&self) -> Vec3 {
        let (sin, cos) = self.yaw.sin_cos();
        vec3(sin, 0.0, -cos)
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y)
    }

    pub fn eye(&self) -> Vec3 {
        self.position + Vec3::Y * EYE
    }

    pub fn look(&mut self, delta: glam::Vec2) {
        self.yaw += delta.x * LOOK;
        self.pitch = (self.pitch - delta.y * LOOK).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Walks, sliding along whatever is in the way rather than stopping dead.
    pub fn walk(&mut self, wish: Vec3, dt: f32, walls: &[Aabb]) {
        if wish.length_squared() < 1e-6 {
            return;
        }

        let body = Sphere::new(self.position + Vec3::Y * RADIUS, RADIUS);
        let moved = move_and_slide(body, wish.normalize() * SPEED, dt, walls);

        self.position = moved - Vec3::Y * RADIUS;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall_at(x: f32) -> Aabb {
        Aabb::from_center_size(vec3(x, 1.0, 0.0), vec3(0.4, 2.0, 8.0))
    }

    #[test]
    fn a_wall_stops_you() {
        let wall = wall_at(2.0);
        let mut player = Player::at(Vec3::ZERO);

        for _ in 0..60 {
            player.walk(Vec3::X, 0.05, &[wall]);
        }

        assert!(
            player.position.x < 2.0 - 0.4 * 0.5,
            "walked to {} and the wall is at 2",
            player.position.x
        );
    }

    #[test]
    fn a_corner_turns_you() {
        // pushing diagonally into a wall keeps the along-the-wall part
        let wall = wall_at(2.0);
        let mut player = Player::at(Vec3::ZERO);
        let before = player.position.z;

        for _ in 0..40 {
            player.walk(vec3(1.0, 0.0, 1.0), 0.05, &[wall]);
        }

        assert!(
            player.position.z > before + 1.0,
            "slid only {} along the wall",
            player.position.z - before
        );
    }

    #[test]
    fn forward_is_where_you_are_looking() {
        let mut player = Player::at(Vec3::ZERO);
        assert!((player.forward() - vec3(0.0, 0.0, -1.0)).length() < 1e-5);

        player.yaw = std::f32::consts::FRAC_PI_2;
        assert!((player.forward() - vec3(1.0, 0.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn walking_is_flat_however_you_look() {
        // looking at the floor and walking forward must not drive you into it
        let mut player = Player::at(Vec3::ZERO);
        player.pitch = -1.0;

        player.walk(player.forward(), 0.1, &[]);

        assert!(player.position.y.abs() < 1e-5, "{}", player.position.y);
    }

    #[test]
    fn you_cannot_look_past_straight_up() {
        let mut player = Player::at(Vec3::ZERO);

        for _ in 0..500 {
            player.look(glam::vec2(0.0, -100.0));
        }

        assert!(player.pitch <= PITCH_LIMIT + 1e-5, "{}", player.pitch);
        assert!(player.facing().is_finite());
    }
}
