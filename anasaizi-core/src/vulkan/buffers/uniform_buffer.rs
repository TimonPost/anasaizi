use crate::{
    engine::RenderContext,
    vulkan::{buffers::buffer::create_allocate_vk_buffer, LogicalDevice},
};
use ash::{version::DeviceV1_0, vk};

/// A uniform buffer that is feeded into the shader.
pub struct UniformBuffer {
    // There is a uniform buffer for each frame.
    buffer: Vec<vk::Buffer>,
    buffers_memory: Vec<vk::DeviceMemory>,
    frames_count: usize,
    pub uniform_object_size: usize,
}

impl UniformBuffer {
    /// Creates a new uniform buffer.
    pub fn new(
        render_context: &RenderContext,
        frames_count: usize,
        buffer_object_sizebuffer_object_size: usize,
    ) -> UniformBuffer {
        let mut buffers = vec![];
        let mut buffers_memory = vec![];

        for _i in 0..frames_count {
            let (buffer, memory) = create_allocate_vk_buffer(
                render_context,
                buffer_object_sizebuffer_object_size as u64,
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
            uniform_object_size: buffer_object_sizebuffer_object_size,
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
