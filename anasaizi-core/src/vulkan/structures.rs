use crate::vulkan::VkLogicalDevice;
use ash::{version::DeviceV1_0, vk};

use std::ffi::CString;

pub struct VkSyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
}

impl VkSyncObjects {
    pub(crate) unsafe fn destroy(&self, device: &VkLogicalDevice) {
        for i in 0..self.image_available_semaphores.len() {
            device.destroy_semaphore(self.image_available_semaphores[i], None);
            device.destroy_semaphore(self.render_finished_semaphores[i], None);
            device.destroy_fence(self.inflight_fences[i], None);
        }
    }
}

pub struct VkValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

impl VkValidationInfo {
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
