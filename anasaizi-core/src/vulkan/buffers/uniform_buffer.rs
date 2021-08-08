use crate::vulkan::{
    buffers::buffer::create_allocate_vk_buffer, Instance, LogicalDevice, Pipeline,
    UniformBufferObjectTemplate,
};
use ash::{version::DeviceV1_0, vk, vk::CommandBuffer};
use std::{marker::PhantomData, mem::size_of};

/// A uniform buffer that is feeded into the shader.
pub struct UniformBuffer<U: UniformBufferObjectTemplate> {
    // There is a uniform buffer for each frame.
    buffer: Vec<vk::Buffer>,
    buffers_memory: Vec<vk::DeviceMemory>,
    frames_count: usize,

    _data: PhantomData<U>,
}

impl<U: UniformBufferObjectTemplate> UniformBuffer<U> {
    /// Creates a new uniform buffer.
    pub fn new(
        instance: &Instance,
        device: &LogicalDevice,
        frames_count: usize,
    ) -> UniformBuffer<U> {
        let buffer_size = size_of::<U>() as u64;

        let mut buffers = vec![];
        let mut buffers_memory = vec![];

        for _i in 0..frames_count {
            let (buffer, memory) = create_allocate_vk_buffer(
                instance,
                device,
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
            buffers.push(buffer);
            buffers_memory.push(memory);
        }

        UniformBuffer {
            buffer: buffers,
            buffers_memory,
            frames_count,

            _data: PhantomData,
        }
    }

    pub fn destroy(&self, device: &LogicalDevice) {
        for i in 0..self.frames_count {
            unsafe {
                device.destroy_buffer(self.buffer[i], None);
                device.free_memory(self.buffers_memory[i], None);
            }
        }
    }

    pub fn buffers_memory(&self, image_index: usize) -> vk::DeviceMemory {
        self.buffers_memory[image_index]
    }

    pub fn buffers(&self, image_index: usize) -> vk::Buffer {
        self.buffer[image_index]
    }
}
