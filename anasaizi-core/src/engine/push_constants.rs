use nalgebra::Vector4;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy)]
pub struct ObjectIdPushConstants {
    pub color: Vector4<f32>,
    pub model_matrix: nalgebra::Matrix4<f32>,
}

#[derive(Serialize, Clone, Copy)]
pub struct MeshPushConstants {
    pub model_matrix: nalgebra::Matrix4<f32>,
    pub texture_id: i32,
}

#[derive(Serialize, Clone, Copy)]
pub struct PBRMeshPushConstants {
    pub model_matrix: nalgebra::Matrix4<f32>,
    pub albedo_map: i32,
    pub normal_map: i32,
    pub metallic_map: i32,
    pub roughness_map: i32,
    pub ao_map: i32,
    pub displacement_map: i32,
}

#[derive(Default, Debug, Serialize, Clone, Copy)]
pub struct GLTFMaterial {
    pub model_matrix: nalgebra::Matrix4<f32>,

    pub base_color_factor: Vector4<f32>,
    pub metallic_roughness_values: Vector4<f32>,
    pub emissive_factor: Vector4<f32>,
    pub scale_ibl_ambient: Vector4<f32>,

    pub base_color_texture: i32,
    pub normal_texture: i32,
    pub metallic_roughness_texture: i32,
    pub occlusion_texture: i32,
    pub emissive_texture: i32,

    pub base_color_texture_coord: i32,
    pub normal_texture_coord: i32,
    pub metallic_factor_texture_coord: i32,
    pub occlusion_texture_coord: i32,
    pub emissive_texture_coord: i32,

    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: f32,
}

#[derive(Serialize, Clone, Copy)]
pub struct UIPushConstants {
    pub ortho_matrix: nalgebra::Matrix4<f32>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GltfPBRShaderConstants {
    pub has_basecolormap: u32,
    pub has_normalmap: u32,
    pub has_emissivemap: u32,
    pub has_metalroughnessmap: u32,
    pub has_occlusionmap: u32,
    pub use_ibl: u32,

    pub has_normals: u32,
    pub has_tangents: u32,
    pub has_colors: u32,
    pub has_uvs: u32,

    pub texture_array_lenght: u32,
}
