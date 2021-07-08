use ash::vk;
use ash::vk::{StructureType, SharingMode};
use crate::math::Vertex;
use std::ptr;
use std::mem::size_of;
use crate::vulkan::{LogicalDevice, Instance, Queue, CommandPool};
use ash::version::{DeviceV1_0, InstanceV1_0};
use core::ops::Deref;
use crate::vulkan::buffers::buffer::{create_buffer, copy_buffer};

pub struct VertexBuffer {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory
}

impl VertexBuffer {
    pub fn create(instance: &Instance, device: &LogicalDevice, vertices: &Vec<Vertex>, submit_queue: &Queue, command_pool: &CommandPool) -> VertexBuffer {
        let buffer_size = (size_of::<Vertex>() * vertices.len()) as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            &instance,
            &device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        );

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            device.unmap_memory(staging_buffer_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            &instance,
            &device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL
        );

        copy_buffer(
            device,
            submit_queue,
            command_pool,
            staging_buffer,
            vertex_buffer,
            buffer_size,
        );

        VertexBuffer {
            vertex_buffer,
            vertex_buffer_memory
        }
    }

    pub fn destroy(&self, device: &LogicalDevice) {
        unsafe  {
            device.destroy_buffer(self.vertex_buffer, None);
            device.free_memory(self.vertex_buffer_memory, None)
        }
    }
}

impl Deref for VertexBuffer {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.vertex_buffer
    }
}
