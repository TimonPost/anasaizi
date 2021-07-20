use ash::{version::DeviceV1_0, vk};

use crate::vulkan::LogicalDevice;
use std::ops::Deref;

/// A Vulkan command pool.
/// Command pools manage the memory that is used to store the buffers and command buffers are allocated from them.
pub struct CommandPool {
    pool: vk::CommandPool,
}

impl CommandPool {
    pub fn create(device: &LogicalDevice) -> CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(device.queue_family_indices().graphics_family.unwrap());

        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        };

        CommandPool { pool: command_pool }
    }
}

impl Deref for CommandPool {
    type Target = vk::CommandPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
