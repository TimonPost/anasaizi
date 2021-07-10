use crate::vulkan::{LogicalDevice, UniformBuffer, UniformBufferObject};
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
    ) -> DescriptorSet {
        let descriptor_buffer_info = [vk::DescriptorBufferInfo {
            buffer: uniform_buffer,
            offset: 0,
            range: std::mem::size_of::<UniformBufferObject>() as u64,
        }];

        let descriptor_write_sets = [vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: descriptor_set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            p_image_info: ptr::null(),
            p_buffer_info: descriptor_buffer_info.as_ptr(),
            p_texel_buffer_view: ptr::null(),
        }];

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
        let descriptor_pool = vk::DescriptorPoolSize::builder()
            .descriptor_count(swap_chain_image_count as u32)
            .build();

        let pool = &[descriptor_pool];

        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool)
            .max_sets(swap_chain_image_count as u32);

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

    pub fn create_descriptor_sets(
        &self,
        device: &LogicalDevice,
        uniforms_buffer: &UniformBuffer,
        descriptor_set_layout: vk::DescriptorSetLayout,
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
            ));
        }

        descriptor_set
    }

    pub fn descriptor_set_layout(device: &LogicalDevice) -> vk::DescriptorSetLayout {
        let layout_binding = [vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        }];

        let layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            binding_count: layout_binding.len() as u32,
            p_bindings: layout_binding.as_ptr(),
        };

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_create_info, None)
                .expect("failed to create descriptor set layout!")
        };

        descriptor_set_layout
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
