mod mesh;
mod object;

use crate::math::Vertex;
use nalgebra::{Vector2, Vector3};

pub use mesh::Mesh;
pub use object::Object;

pub fn square_vertices() -> [Vertex; 8] {
    return [
        Vertex {
            pos: Vector3::new(-0.5, -0.5, 0.0),
            color: Vector3::new(1.0, 0.0, 0.0),
            tex_coord: Vector2::new(1.0, 0.0),
        },
        Vertex {
            pos: Vector3::new(0.5, -0.5, 0.0),
            color: Vector3::new(1.0, 0.0, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
        },
        Vertex {
            pos: Vector3::new(0.5, 0.5, 0.0),
            color: Vector3::new(0.0, 1.0, 0.0),
            tex_coord: Vector2::new(0.0, 1.0),
        },
        Vertex {
            pos: Vector3::new(-0.5, 0.5, 0.0),
            color: Vector3::new(0.0, 0.0, 1.0),
            tex_coord: Vector2::new(1.0, 1.0),
        },
        Vertex {
            pos: Vector3::new(-0.5, -0.5, -0.5),
            color: Vector3::new(1.0, 0.0, 0.0),
            tex_coord: Vector2::new(1.0, 0.0),
        },
        Vertex {
            pos: Vector3::new(0.5, -0.5, -0.5),
            color: Vector3::new(1.0, 0.0, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
        },
        Vertex {
            pos: Vector3::new(0.5, 0.5, -0.5),
            color: Vector3::new(0.0, 1.0, 0.0),
            tex_coord: Vector2::new(0.0, 1.0),
        },
        Vertex {
            pos: Vector3::new(-0.5, 0.5, -0.5),
            color: Vector3::new(0.0, 0.0, 1.0),
            tex_coord: Vector2::new(1.0, 1.0),
        },
    ];
}

pub fn square_indices() -> [u16; 12] {
    return [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];
}
