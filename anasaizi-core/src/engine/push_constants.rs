use nalgebra::Vector4;
use crate::engine::UniformObjectTemplate;
use crate::libs::imgui::__core::any::Any;
use std::mem::size_of;

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
    pub albedo_map: i32,
    pub normal_map: i32,
    pub metallic_map: i32,
    pub roughness_map: i32,
    pub ao_map: i32,
    pub displacement_map: i32
}

#[derive(Clone, Default)]
pub struct GlTFPBRMeshPushConstants {
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

impl UniformObjectTemplate for GlTFPBRMeshPushConstants {
    fn size(&self) -> usize {
        return size_of::<GlTFPBRMeshPushConstants>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
pub struct UIPushConstants {
    pub ortho_matrix: nalgebra::Matrix4<f32>,
}
