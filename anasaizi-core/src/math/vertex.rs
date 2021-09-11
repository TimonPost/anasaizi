use nalgebra::{Vector2, Vector3, Vector4};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub color: Vector4<f32>,
    pub tex_coord: Vector2<f32>,
    pub normal: Vector3<f32>,
}

#[derive(Clone)]
pub struct PosOnlyVertex {
    pub pos: Vector3<f32>,
}
