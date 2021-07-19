use crate::{
    math::Vertex,
    vulkan::{
        buffers::buffer::{copy_buffer, create_buffer},
        CommandPool, Instance, LogicalDevice, Queue,
    },
};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
    vk::{SharingMode, StructureType},
};
use core::ops::Deref;
use std::{mem::size_of, ptr};

pub struct IndexBuffer {
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    indices_count: usize,
}

impl IndexBuffer {
    pub fn create(
        instance: &Instance,
        device: &LogicalDevice,
        indices: &Vec<u32>,
        submit_queue: &Queue,
        command_pool: &CommandPool,
    ) -> IndexBuffer {
        let buffer_size = (size_of::<u32>() * indices.len()) as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            &instance,
            &device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u32;

            data_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());

            device.unmap_memory(staging_buffer_memory);
        }

        let (index_buffer, index_buffer_memory) = create_buffer(
            &instance,
            &device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        copy_buffer(
            device,
            submit_queue,
            command_pool,
            staging_buffer,
            index_buffer,
            buffer_size,
        );

        // Clean up the staging buffer.
        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None)
        }

        IndexBuffer {
            index_buffer,
            index_buffer_memory,
            indices_count: indices.len(),
        }
    }

    pub fn destroy(&self, device: &LogicalDevice) {
        unsafe {
            device.destroy_buffer(self.index_buffer, None);
            device.free_memory(self.index_buffer_memory, None)
        }
    }

    pub fn indices_count(&self) -> usize {
        self.indices_count
    }
}

impl Deref for IndexBuffer {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.index_buffer
    }
}
