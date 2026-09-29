//! A candle: a stub of wax with a flame above it. See
//! `specs/0002-walking-it.md`.
//!
//! A sphere with the light at its centre lights none of itself, because every
//! normal points away from the light, and it shadows itself into the bargain.
//! A candle does not have that problem: the flame sits above the wax, so the
//! wax is lit by it the way everything else is.

use blitzkit::mesh::{MeshData, Vertex};
use glam::Vec3;

/// How thick the wax is, and how tall.
pub const WAX_RADIUS: f32 = 0.055;
pub const WAX_HEIGHT: f32 = 0.30;

/// How big the flame is, and how far above the wax it sits.
pub const FLAME_RADIUS: f32 = 0.045;
pub const FLAME_LIFT: f32 = 0.055;

/// Where the light belongs, measured from the foot of the candle.
pub const FLAME_HEIGHT: f32 = WAX_HEIGHT + FLAME_LIFT + FLAME_RADIUS;

/// The wax, as a closed cylinder standing on the origin.
///
/// Closed matters: an open tube is see-through from the side and the candle
/// looks hollow, which is what the first one did.
pub fn wax() -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let sides = 20u32;

    let ring = |y: f32, normal_out: bool| {
        (0..sides)
            .map(|n| {
                let (sin, cos) = (n as f32 / sides as f32 * std::f32::consts::TAU).sin_cos();
                let at = Vec3::new(cos * WAX_RADIUS, y, sin * WAX_RADIUS);
                let normal = if normal_out {
                    Vec3::new(cos, 0.0, sin)
                } else {
                    Vec3::Y * if y > 0.0 { 1.0 } else { -1.0 }
                };
                (at, normal)
            })
            .collect::<Vec<_>>()
    };

    let mut push = |at: Vec3, normal: Vec3| -> u32 {
        vertices.push(Vertex::new(at.to_array(), normal.to_array(), [0.0, 0.0]));
        vertices.len() as u32 - 1
    };

    // the side, as a band of quads
    let low: Vec<u32> = ring(0.0, true)
        .into_iter()
        .map(|(a, n)| push(a, n))
        .collect();
    let high: Vec<u32> = ring(WAX_HEIGHT, true)
        .into_iter()
        .map(|(a, n)| push(a, n))
        .collect();

    for n in 0..sides as usize {
        let next = (n + 1) % sides as usize;
        indices.extend_from_slice(&[low[n], high[n], high[next]]);
        indices.extend_from_slice(&[low[n], high[next], low[next]]);
    }

    // and a lid and a floor, so it is a solid rather than a tube
    for (y, up) in [(WAX_HEIGHT, true), (0.0, false)] {
        let middle = push(
            Vec3::new(0.0, y, 0.0),
            if up { Vec3::Y } else { Vec3::NEG_Y },
        );
        let rim: Vec<u32> = ring(y, false)
            .into_iter()
            .map(|(a, n)| push(a, n))
            .collect();

        for n in 0..sides as usize {
            let next = (n + 1) % sides as usize;
            if up {
                indices.extend_from_slice(&[middle, rim[n], rim[next]]);
            } else {
                indices.extend_from_slice(&[middle, rim[next], rim[n]]);
            }
        }
    }

    MeshData::new(vertices, indices)
}

/// How much taller than wide the flame is drawn.
pub const FLAME_STRETCH: f32 = 2.1;

/// The flame: the engine's sphere, stretched upright. Its sphere is closed and
/// wound the right way round, which a parametric surface of my own was not, and
/// an open surface is what made the first flame look like a shell.
pub fn flame() -> MeshData {
    stretched(
        MeshData::sphere(16, 12),
        Vec3::new(
            FLAME_RADIUS * 2.0,
            FLAME_RADIUS * 2.0 * FLAME_STRETCH,
            FLAME_RADIUS * 2.0,
        ),
    )
}

/// The hand holding it: a fist, wider than it is tall, around the wax.
pub const HAND_SIZE: Vec3 = Vec3::new(0.115, 0.095, 0.145);

/// How far up the wax the hand grips.
pub const HAND_GRIP: f32 = WAX_HEIGHT * 0.34;

pub fn hand() -> MeshData {
    stretched(MeshData::sphere(14, 10), HAND_SIZE)
}

/// A mesh scaled about its own origin, normals included.
///
/// Scaling a mesh non-uniformly and keeping its normals leaves them pointing
/// the wrong way, which lights it wrong; they take the inverse scale.
pub fn stretched(mesh: MeshData, size: Vec3) -> MeshData {
    let vertices = mesh
        .vertices
        .iter()
        .map(|vertex| {
            let at = Vec3::from(vertex.position) * size;
            let normal = (Vec3::from(vertex.normal) / size).normalize_or_zero();
            Vertex::new(at.to_array(), normal.to_array(), vertex.uv)
        })
        .collect();

    MeshData::new(vertices, mesh.indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flame_sits_above_the_wax() {
        // its whole body, not just its middle: a stretched flame is taller than
        // the gap the lift leaves for it if nobody checks
        let top_of_wax = wax().bounds().max.y;
        let bottom_of_flame = FLAME_HEIGHT + flame().bounds().min.y;

        assert!(
            bottom_of_flame > top_of_wax,
            "the flame reaches down to {} and the wax up to {}",
            bottom_of_flame,
            top_of_wax
        );
    }

    #[test]
    fn the_flame_is_upright() {
        let size = flame().bounds().size();

        assert!(
            size.y > size.x,
            "a flame is taller than it is wide: {:?}",
            size
        );
    }

    #[test]
    fn the_hand_is_wide_enough_to_hold_the_wax() {
        let grip = hand().bounds().size();

        assert!(
            grip.x > WAX_RADIUS * 2.0 && grip.z > WAX_RADIUS * 2.0,
            "a hand {:?} cannot close around wax {} across",
            grip,
            WAX_RADIUS * 2.0
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
        let size = wax().bounds().size();

        assert!(size.y > size.x * 2.0, "{:?} is not a candle shape", size);
    }

    #[test]
    fn the_wax_is_closed() {
        // an open tube is see-through from the side and looks hollow. A closed
        // one has every edge shared by exactly two triangles.
        let mesh = wax();
        let mut edges = std::collections::HashMap::new();

        for triangle in mesh.indices.chunks(3) {
            for (a, b) in [
                (triangle[0], triangle[1]),
                (triangle[1], triangle[2]),
                (triangle[2], triangle[0]),
            ] {
                let at = |i: u32| {
                    let p = mesh.vertices[i as usize].position;
                    (
                        (p[0] * 1e4) as i64,
                        (p[1] * 1e4) as i64,
                        (p[2] * 1e4) as i64,
                    )
                };
                let (one, other) = (at(a), at(b));
                let key = if one < other {
                    (one, other)
                } else {
                    (other, one)
                };
                *edges.entry(key).or_insert(0) += 1;
            }
        }

        let open: Vec<_> = edges.iter().filter(|(_, n)| **n != 2).collect();
        assert!(
            open.is_empty(),
            "{} edges are not shared by two faces",
            open.len()
        );
    }
}
