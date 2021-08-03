use ash::{version::DeviceV1_0, vk};

use crate::vulkan::LogicalDevice;
use std::ops::Deref;

/// A Vulkan command pool.
///
/// A command pools manages the command buffer allocation and the associated memory.
pub struct CommandPool {
    command_pool: vk::CommandPool,
}

impl CommandPool {
    /// Creates a new command pool.
    pub fn create(device: &LogicalDevice) -> CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(device.queue_family_indices().graphics_family.unwrap());

        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        };

        CommandPool { command_pool }
    }

    /// Destroys the command pool and the associated command buffer allocations.
    pub unsafe fn destroy(&self, device: &LogicalDevice) {
        device.destroy_command_pool(self.command_pool, None);
    }
}

impl Deref for CommandPool {
    type Target = vk::CommandPool;

    fn deref(&self) -> &Self::Target {
        &self.command_pool
    }
}
