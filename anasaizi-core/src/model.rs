mod mesh;

use crate::math::Vertex;
use nalgebra::{Vector2, Vector3};

pub use mesh::Mesh;

pub fn triangle_vertices() -> [Vertex; 3] {
 return [
      Vertex { pos: Vector2::new(0.0, -0.5), color: Vector3::new(1.0,0.0,0.0) },
      Vertex { pos: Vector2::new(0.5, 0.5), color: Vector3::new(0.0,1.0,0.0) },
      Vertex { pos: Vector2::new(-0.5, 0.5), color: Vector3::new(0.0,0.0,1.0) }
  ]
}

pub fn square_vertices() -> [Vertex; 4] {
    return [
        Vertex { pos: Vector2::new(-0.5, -0.5), color: Vector3::new(1.0,0.0,0.0) },
        Vertex { pos: Vector2::new(0.5, -0.5), color: Vector3::new(1.0,0.0,0.0) },
        Vertex { pos: Vector2::new(0.5, 0.5), color: Vector3::new(0.0,1.0,0.0) },
        Vertex { pos: Vector2::new(-0.5, 0.5), color: Vector3::new(0.0,0.0,1.0) }
    ]
}