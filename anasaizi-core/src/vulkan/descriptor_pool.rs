use crate::vulkan::{
    ImageView, LogicalDevice, UniformBuffer, UniformBufferObject, UniformBufferObjectTemplate,
};
use ash::{version::DeviceV1_0, vk};
use std::{ops::Deref, ptr};

pub struct DescriptorSet {
    descriptor_set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub fn new(
        device: &LogicalDevice,
        descriptor_set: vk::DescriptorSet,
        uniform_buffer: vk::Buffer,
        texture_sampler: vk::Sampler,
        texture_image_view: vk::ImageView,
    ) -> DescriptorSet {
        let descriptor_buffer_info = [vk::DescriptorBufferInfo {
            buffer: uniform_buffer,
            offset: 0,
            range: std::mem::size_of::<UniformBufferObject>() as u64,
        }];

        let descriptor_image_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_image_view)
            .sampler(texture_sampler)
            .build()];

        let descriptor_write_sets = [
            vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&descriptor_buffer_info)
                .dst_array_element(0)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&descriptor_image_info)
                .dst_array_element(0)
                .build(),
        ];

        unsafe {
            device.update_descriptor_sets(&descriptor_write_sets, &[]);
        }

        DescriptorSet { descriptor_set }
    }
}

impl Deref for DescriptorSet {
    type Target = vk::DescriptorSet;

    fn deref(&self) -> &Self::Target {
        &self.descriptor_set
    }
}

/// Descriptor sets can't be created directly, they must be allocated from a pool like command buffers.
/// The equivalent for descriptor sets is unsurprisingly called a descriptor pool.
pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    swap_chain_image_count: usize,
}

impl DescriptorPool {
    pub fn new(device: &LogicalDevice, swap_chain_image_count: usize) -> DescriptorPool {
        let descriptor_pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swap_chain_image_count as u32,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: swap_chain_image_count as u32,
            },
        ];

        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&descriptor_pool_sizes)
            .max_sets(swap_chain_image_count as u32)
            .build();

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&pool_create_info, None)
                .expect("Could not create descriptor pool.")
        };

        DescriptorPool {
            swap_chain_image_count,
            descriptor_pool,
        }
    }

    pub fn create_descriptor_sets<U: UniformBufferObjectTemplate>(
        &self,
        device: &LogicalDevice,
        uniforms_buffer: &UniformBuffer<U>,
        descriptor_set_layout: vk::DescriptorSetLayout,
        texture_sampler: vk::Sampler,
        texture_image_view: ImageView,
    ) -> Vec<DescriptorSet> {
        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];

        for _ in 0..self.swap_chain_image_count {
            layouts.push(descriptor_set_layout);
        }

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: self.descriptor_pool,
            descriptor_set_count: self.swap_chain_image_count as u32,
            p_set_layouts: layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        let mut descriptor_set = vec![];

        for (i, descritptor_set) in descriptor_sets.iter().enumerate() {
            descriptor_set.push(DescriptorSet::new(
                device,
                *descritptor_set,
                uniforms_buffer.buffers(i),
                texture_sampler,
                *texture_image_view,
            ));
        }

        descriptor_set
    }

    pub fn cleanup(&self, device: &LogicalDevice) {
        unsafe {
            device.destroy_descriptor_pool(self.descriptor_pool, None);
        };
    }
}

impl Deref for DescriptorPool {
    type Target = vk::DescriptorPool;

    fn deref(&self) -> &Self::Target {
        &self.descriptor_pool
    }
}
