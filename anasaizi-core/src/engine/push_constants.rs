use nalgebra::Vector4;

pub struct ObjectIdPushConstants {
    pub color: Vector4<f32>,
    pub model_matrix: nalgebra::Matrix4<f32>,
}

pub struct MeshPushConstants {
    pub model_matrix: nalgebra::Matrix4<f32>,
    pub texture_id: i32,
}

pub struct UIPushConstants {
    pub ortho_matrix: nalgebra::Matrix4<f32>,
}