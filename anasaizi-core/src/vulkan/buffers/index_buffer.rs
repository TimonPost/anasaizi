use crate::{
    engine::RenderContext,
    vulkan::buffers::buffer::{copy_buffer, create_allocate_vk_buffer},
};
use ash::{version::DeviceV1_0, vk};
use core::ops::Deref;
use std::{mem, mem::size_of};

/// An allocated vulkan buffer containing indices.
pub struct IndexBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    count: usize,
}

impl IndexBuffer {
    /// Creates a new index buffer from the given indices.
    pub fn create(render_context: &RenderContext, indices: &Vec<u32>) -> IndexBuffer {
        let device = render_context.device();
        // Allocate the staging buffer.
        let buffer_size = (size_of::<u32>() * indices.len()) as u64;

        let (staging_buffer, staging_buffer_memory) = create_allocate_vk_buffer(
            &render_context,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        // Copy the indices into the allocated buffer.
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

        // Create new buffer on the GPU.
        let (index_buffer, index_buffer_memory) = create_allocate_vk_buffer(
            render_context,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_VISIBLE,
        );

        // Copy data from CPU staging buffer to GPU
        copy_buffer(render_context, staging_buffer, index_buffer, buffer_size);

        // Clean up the staging buffer.
        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None)
        }

        IndexBuffer {
            buffer: index_buffer,
            memory: index_buffer_memory,
            count: indices.len(),
        }
    }

    /// Destroys the buffer and its memory.
    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None)
        }
    }

    /// Returns the number of indices.
    pub fn indices_count(&self) -> usize {
        self.count
    }

    /// Updates the buffer contents with the given data.
    ///
    /// Make sure that the given data is the same as what is stored in the buffer.
    pub fn update_buffer_content<T: Copy>(&self, device: &ash::Device, data: &[T]) {
        unsafe {
            let size = (data.len() * mem::size_of::<T>()) as _;

            let data_ptr = device
                .map_memory(self.memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap();
            let mut align = ash::util::Align::new(data_ptr, mem::align_of::<T>() as _, size);
            align.copy_from_slice(&data);
            device.unmap_memory(self.memory);
        };
    }
}

impl Deref for IndexBuffer {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
