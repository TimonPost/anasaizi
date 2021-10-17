use crate::vulkan::{DescriptorSet, UniformBuffer, VkLogicalDevice};
use ash::{version::DeviceV1_0, vk};
use std::{collections::HashMap, ops::Deref, ptr};

/// Descriptor sets can't be created directly, they must be allocated from a pool like command buffers.
/// The equivalent for descriptor sets is unsurprisingly called a descriptor pool.
pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    image_count: usize,
}

impl DescriptorPool {
    /// Creates a new `DescriptorPool`.
    pub fn new(
        device: &ash::Device,
        descriptor_types: &[vk::DescriptorType],
        image_count: usize,
    ) -> DescriptorPool {
        let mut descriptor_pool_sizes = vec![];

        for descriptor_type in descriptor_types {
            descriptor_pool_sizes.push(vk::DescriptorPoolSize {
                ty: *descriptor_type,
                descriptor_count: image_count as u32,
            });
        }

        let mut pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(image_count as u32)
            .build();

        if descriptor_types.len() != 0 {
            pool_create_info.pool_size_count = descriptor_pool_sizes.len() as u32;
            pool_create_info.p_pool_sizes = descriptor_pool_sizes.as_ptr();
        }

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&pool_create_info, None)
                .expect("Could not create descriptor pool.")
        };

        DescriptorPool {
            image_count,
            descriptor_pool,
        }
    }

    /// Creates descriptor sets.
    pub fn create_descriptor_sets(
        &self,
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
        descriptor_write_sets: Vec<vk::WriteDescriptorSet>,
        uniform_buffers: &mut Vec<UniformBuffer>,
    ) -> Vec<DescriptorSet> {
        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];

        for _ in 0..self.image_count {
            layouts.push(descriptor_set_layout);
        }

        // Allocate descriptor sets
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: self.descriptor_pool,
            descriptor_set_count: self.image_count as u32,
            p_set_layouts: layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        // Create descriptor sets.
        let mut descriptor_set = vec![];

        for (i, descritptor_set) in descriptor_sets.iter().enumerate() {
            let mut descriptor_buffer_infos = HashMap::new();

            let mut write_sets = vec![];
            write_sets.extend_from_slice(&descriptor_write_sets);

            let mut uniform_buffer_index = 0;
            for descriptor_write_set in write_sets.iter_mut() {
                if descriptor_write_set.descriptor_type == vk::DescriptorType::UNIFORM_BUFFER {
                    let uniform_buffer = &mut uniform_buffers[uniform_buffer_index];
                    let frame_uniform_buffer = uniform_buffer.buffers(i);

                    descriptor_buffer_infos.insert(
                        uniform_buffer_index,
                        vk::DescriptorBufferInfo {
                            buffer: frame_uniform_buffer,
                            offset: 0,
                            range: uniform_buffer.uniform_object_size as u64,
                        },
                    );

                    let buffer_info = descriptor_buffer_infos
                        .get(&(uniform_buffer_index))
                        .unwrap();

                    descriptor_write_set.p_buffer_info = buffer_info;
                    descriptor_write_set.descriptor_count = 1;

                    uniform_buffer_index += 1;
                }
            }

            descriptor_set.push(DescriptorSet::new(device, *descritptor_set, write_sets));
        }

        descriptor_set
    }

    /// Destroys the descriptor pool and the associated descriptor set memory.
    pub fn destroy(&self, device: &VkLogicalDevice) {
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
