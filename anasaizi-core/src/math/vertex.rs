use nalgebra::{Vector2, Vector3};

#[derive(Clone)]
pub struct Vertex {
    pub pos: Vector2<f32>,
    pub color: Vector3<f32>
}

