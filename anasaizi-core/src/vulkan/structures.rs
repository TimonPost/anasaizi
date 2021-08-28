use crate::vulkan::LogicalDevice;
use ash::{version::DeviceV1_0, vk};
use nalgebra::{Vector4, Vector3};
use std::{ffi::CString, mem::size_of};
use crate::reexports::imgui::__core::any::Any;

#[derive(Copy, Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

pub struct SyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
}

impl SyncObjects {
    pub(crate) unsafe fn destroy(&self, device: &LogicalDevice) {
        for i in 0..self.image_available_semaphores.len() {
            device.destroy_semaphore(self.image_available_semaphores[i], None);
            device.destroy_semaphore(self.render_finished_semaphores[i], None);
            device.destroy_fence(self.inflight_fences[i], None);
        }
    }
}

pub struct ValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

impl ValidationInfo {
    pub fn to_vec_owned(&self) -> Vec<String> {
        return self
            .required_validation_layers
            .map(|l| l.to_string())
            .to_vec();
    }

    pub fn to_vec_ptr(&self) -> Vec<CString> {
        return self
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).expect("Could not parse cstr"))
            .collect();
    }
}

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

/// Template for an uniform buffer object.
pub trait UniformObjectTemplate: UniformObjectClone  {
    /// Returns the size of this buffer object.
    fn size(&self) -> usize;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait UniformObjectClone {
    fn clone_box(&self) -> Box<dyn UniformObjectTemplate>;
}

impl<T> UniformObjectClone for T
    where
        T: 'static + UniformObjectTemplate + Clone+ Default,
{
    fn clone_box(&self) -> Box<dyn UniformObjectTemplate> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn UniformObjectTemplate> {
    fn clone(&self) -> Box<dyn UniformObjectTemplate> {
        self.clone_box()
    }
}

#[derive(Clone, Copy)]
pub struct LightingUniformBufferObject {
    pub shininess:f32,
    pub specular_strength: f32,

    pub ambient_strength: f32,

    pub light_position: Vector3<f32>,
    pub light_color: Vector3<f32>,

    pub view_pos: Vector3<f32>,
}

impl Default for LightingUniformBufferObject {
    fn default() -> Self {
        LightingUniformBufferObject {
            shininess: 32.0,
            specular_strength: 0.5,
            ambient_strength: 0.1,
            light_position: Vector3::default(),
            light_color: Vector3::new(1.0,1.0,1.0),
            view_pos: Vector3::default(),
        }
    }
}

impl UniformObjectTemplate for LightingUniformBufferObject {
    fn size(&self) -> usize {
        size_of::<LightingUniformBufferObject>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Uniform buffer object.
#[derive(Clone, Copy)]
pub struct UniformBufferObject {
    pub view_matrix: nalgebra::Matrix4<f32>,
    pub projection_matrix: nalgebra::Matrix4<f32>,
}

impl UniformObjectTemplate for UniformBufferObject {
    fn size(&self) -> usize {
        size_of::<UniformBufferObject>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for UniformBufferObject {
    fn default() -> Self {
        let mut identity = nalgebra::Matrix4::default();
        identity.fill_with_identity();

        UniformBufferObject {
            view_matrix: identity,
            projection_matrix: identity,
        }
    }
}
