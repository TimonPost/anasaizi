use ash::{version::DeviceV1_0, vk};

use crate::vulkan::VkLogicalDevice;
use std::ops::Deref;

/// A Vulkan command pool.
///
/// A command pools manages the command buffer allocation and the associated memory.
pub struct VkCommandPool {
    command_pool: vk::CommandPool,
}

impl VkCommandPool {
    /// Creates a new command pool.
    pub fn create(device: &VkLogicalDevice) -> VkCommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(device.queue_family_indices().graphics_family.unwrap());

        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        };

        VkCommandPool { command_pool }
    }

    /// Destroys the command pool and the associated command buffer allocations.
    pub unsafe fn destroy(&self, device: &VkLogicalDevice) {
        device.destroy_command_pool(self.command_pool, None);
    }
}

impl Deref for VkCommandPool {
    type Target = vk::CommandPool;

    fn deref(&self) -> &Self::Target {
        &self.command_pool
    }
}
