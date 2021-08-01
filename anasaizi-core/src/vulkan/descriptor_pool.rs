use crate::vulkan::{LogicalDevice, UniformBuffer, UniformBufferObjectTemplate};
use ash::{version::DeviceV1_0, vk};
use std::{ops::Deref, ptr};

pub struct DescriptorSet {
    descriptor_set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub fn new(
        device: &LogicalDevice,
        descriptor_set: vk::DescriptorSet,
        descriptor_write_sets: &mut Vec<vk::WriteDescriptorSet>,
        descriptor_info: &[vk::DescriptorBufferInfo],
    ) -> DescriptorSet {
        for mut descriptor in descriptor_write_sets.iter_mut() {
            descriptor.dst_set = descriptor_set;
            descriptor.p_buffer_info = descriptor_info.as_ptr();
            descriptor.descriptor_count = descriptor_info.len() as u32
        }

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
    pub fn new(
        device: &LogicalDevice,
        descriptor_types: &[vk::DescriptorType],
        swap_chain_image_count: usize,
    ) -> DescriptorPool {
        let mut descriptor_pool_sizes = vec![];

        for descriptor_type in descriptor_types {
            descriptor_pool_sizes.push(vk::DescriptorPoolSize {
                ty: *descriptor_type,
                descriptor_count: swap_chain_image_count as u32,
            });
        }

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
        descriptor_set_layout: vk::DescriptorSetLayout,
        mut descriptor_write_sets: Vec<vk::WriteDescriptorSet>,
        uniform_buffer: &UniformBuffer<U>,
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
            let descriptor_buffer_info = [vk::DescriptorBufferInfo {
                buffer: uniform_buffer.buffers(i),
                offset: 0,
                range: std::mem::size_of::<U>() as u64,
            }];

            descriptor_set.push(DescriptorSet::new(
                device,
                *descritptor_set,
                &mut descriptor_write_sets,
                &descriptor_buffer_info,
            ));
        }

        descriptor_set
    }

    pub fn destroy(&self, device: &LogicalDevice) {
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
