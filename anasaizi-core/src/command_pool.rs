use crate::structures::QueueFamilyIndices;
use ash::vk;
use ash::version::DeviceV1_0;
use std::ptr;
use std::process::Command;
use std::ops::Deref;
use crate::LogicalDevice;

pub struct CommandPool {
    pool: vk::CommandPool
}

impl CommandPool {
    pub fn create(
        device: &LogicalDevice
    ) -> CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::empty(),
            queue_family_index: device.queue_family_indices().graphics_family.unwrap(),
        };

        let command_pool = unsafe {
            device
                .logical_device()
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        };

        CommandPool {
            pool: command_pool
        }
    }
}

impl Deref for CommandPool {
    type Target = vk::CommandPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}