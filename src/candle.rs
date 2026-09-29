//! A candle: a stub of wax with a flame above it. See
//! `specs/0002-walking-it.md`.
//!
//! A sphere with the light at its centre lights none of itself, because every
//! normal points away from the light, and it shadows itself into the bargain.
//! A candle does not have that problem: the flame sits above the wax, so the
//! wax is lit by it the way everything else is.

use blitzkit::mesh::MeshData;
use glam::Vec3;

/// How thick the wax is, and how tall.
pub const WAX_RADIUS: f32 = 0.055;
pub const WAX_HEIGHT: f32 = 0.30;

/// How big the flame is, and how far above the wax it sits.
pub const FLAME_RADIUS: f32 = 0.045;
pub const FLAME_LIFT: f32 = 0.055;

/// Where the light belongs, measured from the foot of the candle.
pub const FLAME_HEIGHT: f32 = WAX_HEIGHT + FLAME_LIFT + FLAME_RADIUS;

/// The wax, as a cylinder standing on the origin.
pub fn wax() -> MeshData {
    MeshData::surface(20, 2, |u, v| {
        let (sin, cos) = (u * std::f32::consts::TAU).sin_cos();
        Vec3::new(cos * WAX_RADIUS, v * WAX_HEIGHT, sin * WAX_RADIUS)
    })
}

/// The flame, as a teardrop: a sphere stretched upwards.
pub fn flame() -> MeshData {
    MeshData::surface(16, 12, |u, v| {
        let (sin_u, cos_u) = (u * std::f32::consts::TAU).sin_cos();
        let angle = v * std::f32::consts::PI;
        let (sin_v, cos_v) = angle.sin_cos();

        // narrower towards the top, which is what makes it a flame rather than
        // a bead
        let taper = 1.0 - v * 0.55;

        Vec3::new(
            cos_u * sin_v * FLAME_RADIUS * taper,
            -cos_v * FLAME_RADIUS * 1.9,
            sin_u * sin_v * FLAME_RADIUS * taper,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flame_sits_above_the_wax() {
        let top_of_wax = wax().bounds().max.y;

        assert!(
            FLAME_HEIGHT > top_of_wax,
            "the flame at {} is inside the wax, which reaches {}",
            FLAME_HEIGHT,
            top_of_wax
        );
    }

    #[test]
    fn the_wax_stands_on_its_foot() {
        let bounds = wax().bounds();

        assert!(bounds.min.y.abs() < 1e-5, "it starts at {}", bounds.min.y);
        assert!((bounds.max.y - WAX_HEIGHT).abs() < 1e-5);
    }

    #[test]
    fn a_candle_is_taller_than_it_is_wide() {
        let bounds = wax().bounds();
        let size = bounds.size();

        assert!(size.y > size.x * 2.0, "{:?} is not a candle shape", size);
    }

    #[test]
    fn the_flame_is_taller_than_it_is_wide() {
        let size = flame().bounds().size();

        assert!(size.y > size.x, "a flame points up: {:?}", size);
    }
}
