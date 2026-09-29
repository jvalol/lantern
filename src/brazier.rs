//! The lights already burning in the maze, and the things that burn them. See
//! `specs/0002-walking-it.md`.
//!
//! These cannot cast, because spec 0022 allows two shadowing point lights and
//! both candles spend them. A light that cannot cast cannot be hidden by a
//! wall, so the only thing that stops one lighting the corridor next door is
//! its range running out first. That is what decides every number here.

use blitzkit::collision::Aabb;
use blitzkit::mesh::MeshData;
use glam::{vec3, Vec3};

/// The stem, and how far up it the bowl sits.
const STEM_HALF: f32 = 0.05;
const STEM_TOP: f32 = 0.58;

/// The bowl: a squashed sphere, which is closed, rather than an open dish,
/// which would look hollow from above.
const BOWL_AT: f32 = 0.64;
const BOWL: Vec3 = vec3(0.30, 0.22, 0.30);

/// Where its flame sits, and so where its light comes from.
pub const FLAME_AT: f32 = 0.84;

/// How big that flame is drawn, a little larger than a candle's.
const FLAME: Vec3 = vec3(0.075, 0.15, 0.075);

/// How wide it stands, for the box that stops you walking through it.
const SOLID_HALF: f32 = 0.16;

/// The stem and the bowl as one mesh, standing on the origin.
pub fn stand() -> MeshData {
    let stem = crate::candle::stretched(
        MeshData::cube(),
        vec3(STEM_HALF * 2.0, STEM_TOP, STEM_HALF * 2.0),
    );
    let bowl = crate::candle::stretched(MeshData::sphere(16, 10), BOWL);

    let mut vertices: Vec<_> = stem
        .vertices
        .iter()
        .map(|v| {
            let at = Vec3::from(v.position) + Vec3::Y * STEM_TOP * 0.5;
            blitzkit::mesh::Vertex::new(at.to_array(), v.normal, v.uv)
        })
        .collect();
    let mut indices = stem.indices.clone();
    let first = vertices.len() as u32;

    vertices.extend(bowl.vertices.iter().map(|v| {
        let at = Vec3::from(v.position) + Vec3::Y * BOWL_AT;
        blitzkit::mesh::Vertex::new(at.to_array(), v.normal, v.uv)
    }));
    indices.extend(bowl.indices.iter().map(|i| i + first));

    MeshData::new(vertices, indices)
}

/// Its flame, drawn at `FLAME_AT`.
pub fn flame() -> MeshData {
    crate::candle::stretched(MeshData::sphere(16, 12), FLAME)
}

/// The box that stops you walking through one.
pub fn solid(at: Vec3) -> Aabb {
    Aabb::from_center_size(
        at + Vec3::Y * STEM_TOP * 0.5,
        vec3(SOLID_HALF * 2.0, STEM_TOP, SOLID_HALF * 2.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_stands_on_the_floor() {
        assert!(stand().bounds().min.y.abs() < 1e-5);
    }

    #[test]
    fn its_flame_clears_its_bowl() {
        let bowl_top = stand().bounds().max.y;
        let flame_bottom = FLAME_AT + flame().bounds().min.y;

        assert!(
            flame_bottom > bowl_top,
            "the flame reaches to {} and the bowl to {}",
            flame_bottom,
            bowl_top
        );
    }

    #[test]
    fn you_can_walk_past_one_in_a_corridor() {
        // it stands in the middle of a cell, so what is left either side has to
        // be wider than you are
        let gap = crate::walls::CELL * 0.5 - crate::walls::THICK * 0.5 - SOLID_HALF;

        assert!(
            gap > crate::player::RADIUS * 2.0,
            "only {} to squeeze through and you are {} across",
            gap,
            crate::player::RADIUS * 2.0
        );
    }
}
