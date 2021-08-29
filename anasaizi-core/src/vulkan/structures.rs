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
