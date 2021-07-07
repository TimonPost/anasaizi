use ash::{version::DeviceV1_0, vk};
use std::ptr;

use crate::vulkan::LogicalDevice;
use std::ops::Deref;

pub struct CommandPool {
    pool: vk::CommandPool,
}

impl CommandPool {
    pub fn create(device: &LogicalDevice) -> CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::empty(),
            queue_family_index: device.queue_family_indices().graphics_family.unwrap(),
        };

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
