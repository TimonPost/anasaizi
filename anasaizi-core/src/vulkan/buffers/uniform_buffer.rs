use crate::vulkan::{buffers::buffer::create_buffer, Instance, LogicalDevice};
use ash::{version::DeviceV1_0, vk};
use std::{marker::PhantomData, mem::size_of, ptr};

pub trait UniformBufferObjectTemplate: Default {
    fn size(&self) -> usize;
}

#[derive(Copy, Clone)]
pub struct UniformBufferObject {
    pub model: nalgebra::Matrix4<f32>,
    pub view: nalgebra::Matrix4<f32>,
    pub proj: nalgebra::Matrix4<f32>,
}

impl UniformBufferObjectTemplate for UniformBufferObject {
    fn size(&self) -> usize {
        size_of::<UniformBufferObject>()
    }
}

impl Default for UniformBufferObject {
    fn default() -> Self {
        UniformBufferObject {
            model: nalgebra::Matrix::default(),
            view: nalgebra::Matrix::default(),
            proj: nalgebra::Matrix::default(),
        }
    }
}

pub struct UniformBuffer<U: UniformBufferObjectTemplate> {
    // because we can pregenerate frames and uniforms change per frame we want to have various buffers in our arsenal to use.
    buffers: Vec<vk::Buffer>,
    buffers_memory: Vec<vk::DeviceMemory>,
    swap_chain_image_count: usize,

    _data: PhantomData<U>,
}

impl<U: UniformBufferObjectTemplate> UniformBuffer<U> {
    pub fn new(
        instance: &Instance,
        device: &LogicalDevice,
        swap_chain_image_count: usize,
    ) -> UniformBuffer<U> {
        let buffer_size = size_of::<U>() as u64;

        let mut buffers = vec![];
        let mut buffers_memory = vec![];

        for i in 0..swap_chain_image_count {
            unsafe {
                let (buffer, memory) = create_buffer(
                    instance,
                    device,
                    buffer_size,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                );
                buffers.push(buffer);
                buffers_memory.push(memory);
            }
        }

        UniformBuffer {
            buffers,
            buffers_memory,
            swap_chain_image_count,

            _data: PhantomData,
        }
    }

    pub fn cleanup(&self, device: &LogicalDevice) {
        for i in 0..self.swap_chain_image_count {
            unsafe {
                device.destroy_buffer(self.buffers[i], None);
                device.free_memory(self.buffers_memory[i], None);
            }
        }
    }

    pub fn buffers_memory(&self, image_index: usize) -> vk::DeviceMemory {
        self.buffers_memory[image_index]
    }

    pub fn buffers(&self, image_index: usize) -> vk::Buffer {
        self.buffers[image_index]
    }
}
