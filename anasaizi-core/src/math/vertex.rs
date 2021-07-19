use nalgebra::{Vector2, Vector3};

#[derive(Clone)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub color: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}
