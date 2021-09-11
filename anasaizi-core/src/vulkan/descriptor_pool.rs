use crate::{
    engine::{
        image::Texture, BufferLayout, RenderContext, UniformObjectTemplate, VulkanApplication,
    },
    libs::imgui::__core::option::IterMut,
    vulkan::{LogicalDevice, UniformBuffer},
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{
        PipelineLayout, PipelineVertexInputStateCreateInfo,
        PipelineVertexInputStateCreateInfoBuilder, PushConstantRange, ShaderStageFlags,
    },
};
use std::{collections::HashMap, mem, ops::Deref, ptr};

/// Think of a single descriptor as a handle or pointer into a resource.
/// That resource being a Buffer or a Image, and also holds other information, such as the size of the buffer, or the type of sampler if it’s for an image.
/// A VkDescriptorSet is a pack of those pointers that are bound together.
pub struct DescriptorSet {
    descriptor_set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub fn new(
        device: &ash::Device,
        descriptor_set: vk::DescriptorSet,
        mut descriptor_write_sets: Vec<vk::WriteDescriptorSet>,
    ) -> DescriptorSet {
        let mut descriptor_write_sets = descriptor_write_sets;
        for descriptor_write_set in descriptor_write_sets.iter_mut() {
            descriptor_write_set.dst_set = descriptor_set;
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
                        (uniform_buffer_index),
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

pub struct ShaderIOBuilder {
    descriptor_types: Vec<vk::DescriptorType>,
    descriptor_layout_bindingen: Vec<vk::DescriptorSetLayoutBinding>,

    uniform_buffers: Vec<UniformBuffer>,

    write_descriptor_sets: Vec<vk::WriteDescriptorSet>,

    input_buffer_layout: Option<BufferLayout>,
    push_constant_ranges: Vec<vk::PushConstantRange>,

    descriptor_image_info: Vec<vk::DescriptorImageInfo>,
    dynamic_descriptor_image_info: Vec<vk::DescriptorImageInfo>,
    sampler: Vec<vk::DescriptorImageInfo>,
}

impl ShaderIOBuilder {
    pub fn builder() -> Self {
        ShaderIOBuilder {
            descriptor_layout_bindingen: vec![],
            descriptor_types: vec![],
            write_descriptor_sets: vec![],
            input_buffer_layout: None,
            push_constant_ranges: vec![],

            // used to keep pointer alive.
            descriptor_image_info: vec![],
            dynamic_descriptor_image_info: vec![],
            sampler: vec![],
            uniform_buffers: vec![],
        }
    }

    /// Shader with the given input buffer.
    pub fn add_input_buffer_layout(mut self, input_buffer_layout: BufferLayout) -> ShaderIOBuilder {
        self.input_buffer_layout = Some(input_buffer_layout);
        self
    }

    pub fn sampler(
        mut self,
        binding_id: u32,
        stage_flags: vk::ShaderStageFlags,
        sampler: vk::Sampler,
    ) -> ShaderIOBuilder {
        let descriptor_type = vk::DescriptorType::SAMPLER;

        self.sampler.push(
            vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .sampler(sampler)
                .build(),
        );

        self.write_descriptor_sets.push(
            vk::WriteDescriptorSet::builder()
                .dst_binding(binding_id)
                .descriptor_type(descriptor_type)
                .dst_array_element(0)
                .image_info(&self.sampler)
                .build(),
        );

        self.descriptor_layout_bindingen.push(
            vk::DescriptorSetLayoutBinding::builder()
                .descriptor_type(descriptor_type)
                .descriptor_count(1) // update texture count
                .stage_flags(stage_flags) //
                .binding(binding_id)
                .build(),
        );

        self.descriptor_types.push(descriptor_type);

        self
    }

    pub fn texture_array(
        mut self,
        binding_id: u32,
        stage_flags: vk::ShaderStageFlags,
        textures: &[Texture],
        sampler: vk::Sampler,
    ) -> ShaderIOBuilder {
        let descriptor_type = vk::DescriptorType::COMBINED_IMAGE_SAMPLER;

        for texture in textures.iter() {
            self.dynamic_descriptor_image_info.push(
                vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(*texture.image_view)
                    .sampler(sampler)
                    .build(),
            );
        }

        self.write_descriptor_sets.push(
            vk::WriteDescriptorSet::builder()
                .dst_binding(binding_id)
                .descriptor_type(descriptor_type)
                .dst_array_element(0)
                .image_info(&self.dynamic_descriptor_image_info)
                .build(),
        );

        self.descriptor_layout_bindingen.push(
            vk::DescriptorSetLayoutBinding::builder()
                .descriptor_type(descriptor_type)
                .descriptor_count(textures.len() as u32) // update texture count
                .stage_flags(stage_flags) //
                .binding(binding_id)
                .build(),
        );

        self.descriptor_types.push(descriptor_type);

        self
    }

    pub fn add_static_image(
        mut self,
        binding_id: u32,
        stage_flags: vk::ShaderStageFlags,
        texture: &Texture,
        sampler: vk::Sampler,
    ) -> ShaderIOBuilder {
        let descriptor_type = vk::DescriptorType::COMBINED_IMAGE_SAMPLER;

        self.descriptor_image_info.push(
            vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(*texture.image_view)
                .sampler(sampler)
                .build(),
        );

        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_binding(binding_id)
            .descriptor_type(descriptor_type)
            .dst_array_element(0)
            .image_info(&self.descriptor_image_info)
            .build();

        let descriptor_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .descriptor_type(descriptor_type)
            .descriptor_count(1)
            .stage_flags(stage_flags)
            .binding(binding_id)
            .build();

        self.write_descriptor_sets.push(write_descriptor_set);
        self.descriptor_layout_bindingen
            .push(descriptor_layout_binding);
        self.descriptor_types.push(descriptor_type);
        self
    }

    pub fn add_uniform_buffer(
        mut self,
        binding_id: u32,
        stage_flags: ShaderStageFlags,
        render_context: &RenderContext,
        frames: usize,
        buffer_object_size: usize,
    ) -> ShaderIOBuilder {
        let buffer = UniformBuffer::new(render_context, frames, buffer_object_size);
        let descriptor_type = vk::DescriptorType::UNIFORM_BUFFER;

        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_binding(binding_id)
            .descriptor_type(descriptor_type)
            .dst_array_element(0)
            .build();

        let descriptor_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .descriptor_type(descriptor_type)
            .descriptor_count(1)
            .stage_flags(stage_flags)
            .binding(binding_id)
            .build();

        self.write_descriptor_sets.push(write_descriptor_set);
        self.descriptor_layout_bindingen
            .push(descriptor_layout_binding);
        self.descriptor_types.push(descriptor_type);

        self.uniform_buffers.push(buffer);
        self
    }

    pub fn add_push_constant_ranges(
        mut self,
        push_constant_ranges: &[PushConstantRange],
    ) -> ShaderIOBuilder {
        self.push_constant_ranges
            .extend_from_slice(push_constant_ranges);
        self
    }

    pub fn build(mut self, render_context: &RenderContext, frames: usize) -> ShaderIo {
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&self.descriptor_layout_bindingen)
            .build();

        let descriptor_set_layout = unsafe {
            render_context
                .device()
                .create_descriptor_set_layout(&layout_create_info, None)
                .expect("failed to create descriptor set layout!")
        };

        self.descriptor_types
            .push(vk::DescriptorType::INPUT_ATTACHMENT); // TODO: this is not required I think+

        let descriptor_pool =
            DescriptorPool::new(&render_context.device(), &self.descriptor_types, frames);

        let descriptor_sets = descriptor_pool.create_descriptor_sets(
            &render_context.device(),
            descriptor_set_layout,
            self.write_descriptor_sets,
            &mut self.uniform_buffers,
        );

        ShaderIo {
            descriptor_pool,
            descriptor_sets,
            uniform_buffer_objects: Vec::with_capacity(self.uniform_buffers.len()),
            uniform_buffers: self.uniform_buffers,
            descriptor_set_layout,
            input_buffer_layout: self.input_buffer_layout.unwrap(),
            push_constant_ranges: self.push_constant_ranges,
        }
    }
}

pub struct ShaderIo {
    pub descriptor_pool: DescriptorPool,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub uniform_buffers: Vec<UniformBuffer>,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub input_buffer_layout: BufferLayout,
    pub push_constant_ranges: Vec<vk::PushConstantRange>,
    pub uniform_buffer_objects: Vec<Box<dyn UniformObjectTemplate>>,
}

impl ShaderIo {
    pub unsafe fn destroy(&self, device: &LogicalDevice) {
        self.descriptor_pool.destroy(device);
        for buffer in &self.uniform_buffers {
            buffer.destroy(device);
        }
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }

    pub fn create_pipeline_layout(&self, device: &LogicalDevice) -> PipelineLayout {
        let descriptor_set_layouts = [self.descriptor_set_layout];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&self.push_constant_ranges)
            .build();

        let pipeline_layout = unsafe {
            device
                .logical_device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout!")
        };

        pipeline_layout
    }

    pub fn vertex_input_info(&mut self) -> PipelineVertexInputStateCreateInfo {
        self.input_buffer_layout.build_attrib_description();
        self.input_buffer_layout.build_binding_description();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&self.input_buffer_layout.binding_desc)
            .vertex_attribute_descriptions(&self.input_buffer_layout.attrib_desc)
            .build();

        vertex_input_info
    }
}
