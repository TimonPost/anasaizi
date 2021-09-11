use nalgebra::Vector4;

pub struct ObjectIdPushConstants {
    pub color: Vector4<f32>,
    pub model_matrix: nalgebra::Matrix4<f32>,
}

pub struct MeshPushConstants {
    pub model_matrix: nalgebra::Matrix4<f32>,
    pub texture_id: i32,
}

pub struct PBRMeshPushConstants {
    pub model_matrix: nalgebra::Matrix4<f32>,
    pub albedo_map: u32,
    pub normal_map: u32,
    pub metallic_map: u32,
    pub roughness_map: u32,
    pub ao_map: u32,
}

pub struct UIPushConstants {
    pub ortho_matrix: nalgebra::Matrix4<f32>,
}
