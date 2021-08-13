use crate::{
    engine::{image::Texture, BufferLayout, RenderContext, VulkanApplication},
    reexports::imgui::__core::option::IterMut,
    vulkan::{LogicalDevice, MeshPushConstants, UniformBuffer, UniformBufferObjectTemplate},
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
use ultraviolet::Mat4;

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
        descriptor_write_sets: Vec<vk::WriteDescriptorSet>,
        descriptor_info: &[vk::DescriptorBufferInfo],
    ) -> DescriptorSet {
        let mut descriptor_write_sets = descriptor_write_sets;
        for mut descriptor in descriptor_write_sets.iter_mut() {
            if descriptor.descriptor_type == vk::DescriptorType::UNIFORM_BUFFER {
                descriptor.p_buffer_info = descriptor_info.as_ptr();
                descriptor.descriptor_count = descriptor_info.len() as u32
            }
            descriptor.dst_set = descriptor_set;
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
    /// Creates a new `DescriptorPool`.
    pub fn new(
        device: &ash::Device,
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

    /// Creates descriptor sets.
    pub fn create_descriptor_sets<U: UniformBufferObjectTemplate>(
        &self,
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
        descriptor_write_sets: Vec<vk::WriteDescriptorSet>,
        uniform_buffer: Option<&UniformBuffer<U>>,
    ) -> Vec<DescriptorSet> {
        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];

        for _ in 0..self.swap_chain_image_count {
            layouts.push(descriptor_set_layout);
        }

        // Allocate descriptor sets
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

        // Create descriptor sets.
        let mut descriptor_set = vec![];

        for (i, descritptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_info = {
                if let Some(uniform_buffer) = uniform_buffer {
                    vec![vk::DescriptorBufferInfo {
                        buffer: uniform_buffer.buffers(i),
                        offset: 0,
                        range: std::mem::size_of::<U>() as u64,
                    }]
                } else {
                    vec![]
                }
            };

            let mut write_sets = vec![];
            write_sets.extend_from_slice(&descriptor_write_sets);

            descriptor_set.push(DescriptorSet::new(
                device,
                *descritptor_set,
                write_sets,
                &descriptor_buffer_info,
            ));
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
                .descriptor_count(2) // update texture count
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
    ) -> ShaderIOBuilder {
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

    pub fn build<U: UniformBufferObjectTemplate>(
        self,
        render_context: &RenderContext,
        frames: usize,
    ) -> ShaderIo<U> {
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&self.descriptor_layout_bindingen)
            .build();

        let descriptor_set_layout = unsafe {
            render_context
                .device()
                .create_descriptor_set_layout(&layout_create_info, None)
                .expect("failed to create descriptor set layout!")
        };

        let uniform_buffer = UniformBuffer::<U>::new(&render_context, frames);

        let descriptor_pool =
            DescriptorPool::new(&render_context.device(), &self.descriptor_types, frames);

        let descriptor_sets = descriptor_pool.create_descriptor_sets::<U>(
            &render_context.device(),
            descriptor_set_layout,
            self.write_descriptor_sets,
            Some(&uniform_buffer),
        );

        let mut texture_descriptor_sets = TextureDescriptorSets::new();

        // for dynamic_texture in self.dynamic_textures.iter() {
        //     let descriptor_layout_bindingen = [dynamic_texture.binding_info];
        //     let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        //         .bindings(&descriptor_layout_bindingen)
        //         .build();
        //
        //     let descriptor_set_layout = unsafe {
        //         application
        //             .device
        //             .create_descriptor_set_layout(&layout_create_info, None)
        //             .expect("failed to create descriptor set layout!")
        //     };
        //
        //     let writes = vec![dynamic_texture.write_descriptor];
        //     let dynamic_texture_descriptor_sets = descriptor_pool.create_descriptor_sets::<U>(
        //         &application.device,
        //         descriptor_set_layout,
        //         writes,
        //         None
        //     );
        //
        //     texture_descriptor_sets.set_texture(dynamic_texture.texture_id.clone(), dynamic_texture_descriptor_sets);
        //}

        ShaderIo {
            descriptor_pool,
            descriptor_sets,
            uniform_buffer,
            descriptor_set_layout,
            input_buffer_layout: self.input_buffer_layout.unwrap(),
            push_constant_ranges: self.push_constant_ranges,
            uniform_buffer_object: Default::default(),
            texture_descriptor_sets,
        }
    }
}

pub struct ShaderIo<U: UniformBufferObjectTemplate> {
    pub descriptor_pool: DescriptorPool,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub uniform_buffer: UniformBuffer<U>,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub input_buffer_layout: BufferLayout,
    pub push_constant_ranges: Vec<vk::PushConstantRange>,
    pub uniform_buffer_object: U,

    pub texture_descriptor_sets: TextureDescriptorSets,
}

impl<U: UniformBufferObjectTemplate> ShaderIo<U> {
    pub unsafe fn destroy(&self, device: &LogicalDevice) {
        self.descriptor_pool.destroy(device);
        self.uniform_buffer.destroy(device);
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

pub struct TextureDescriptorSets {
    descriptor_sets: HashMap<String, Vec<DescriptorSet>>,
}

impl TextureDescriptorSets {
    pub fn new() -> TextureDescriptorSets {
        TextureDescriptorSets {
            descriptor_sets: HashMap::new(),
        }
    }

    pub fn texture(&self, id: String, frame: usize) -> Option<&vk::DescriptorSet> {
        if let Some(sets) = self.descriptor_sets.get(&id) {
            return Some(&sets[frame]);
        }

        None
    }

    pub fn set_texture(&mut self, id: String, descriptor_set: Vec<DescriptorSet>) {
        self.descriptor_sets.insert(id, descriptor_set);
    }
}
