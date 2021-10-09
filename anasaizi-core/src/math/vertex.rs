use nalgebra::{Vector2, Vector3, Vector4};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub color: Vector4<f32>,
    pub tex_coord: Vector2<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub bitangent: Vector3<f32>,
}

#[derive(Clone)]
pub struct PosOnlyVertex {
    pub pos: Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct GltsVertex {
    pub position: Vector4<f32>,
    pub normal: Vector4<f32>,
    pub tangent: Vector4<f32>,
    pub tex_coord_0: Vector2<f32>,
    pub tex_coord_1: Vector2<f32>,
    pub color_0: Vector4<f32>,
}

impl Default for GltsVertex {
    fn default() -> Self {
        GltsVertex {
            position: Vector4::zeros(),
            normal: Vector4::zeros(),
            tangent: Vector4::zeros(),
            tex_coord_0: Vector2::zeros(),
            tex_coord_1: Vector2::zeros(),
            color_0: Vector4::zeros(),
        }
    }
}