use crate::{
    engine::{
        image::Texture, BufferLayout, GltfPBRShaderConstants, RenderContext, UniformObjectTemplate,
    },
    vulkan::{DescriptorPool, DescriptorSet, UniformBuffer, VkLogicalDevice},
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{PipelineLayout, PipelineVertexInputStateCreateInfo, PushConstantRange, ShaderStageFlags},
};

pub struct ShaderIOBuilder {
    descriptor_types: Vec<vk::DescriptorType>,
    descriptor_layout_bindingen: Vec<vk::DescriptorSetLayoutBinding>,

    uniform_buffers: Vec<UniformBuffer>,

    write_descriptor_sets: Vec<vk::WriteDescriptorSet>,

    input_buffer_layout: Option<BufferLayout>,
    push_constant_ranges: Vec<vk::PushConstantRange>,

    specialization_constants: Vec<vk::SpecializationMapEntry>,
    specialization_constant_data: GltfPBRShaderConstants,

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
            specialization_constants: vec![],
            specialization_constant_data: Default::default(),

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

    pub fn add_specialization_constants(
        mut self,
        constant_data: GltfPBRShaderConstants,
        entries: Vec<vk::SpecializationMapEntry>,
    ) -> ShaderIOBuilder {
        self.specialization_constants = entries;
        self.specialization_constant_data = constant_data;

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
            specialization_constants: self.specialization_constants,
            specialization_constant_data: self.specialization_constant_data,
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
    pub specialization_constants: Vec<vk::SpecializationMapEntry>,
    pub specialization_constant_data: GltfPBRShaderConstants,
}

impl ShaderIo {
    pub unsafe fn destroy(&self, device: &VkLogicalDevice) {
        self.descriptor_pool.destroy(device);
        for buffer in &self.uniform_buffers {
            buffer.destroy(device);
        }
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }

    pub fn specialization_data_ref(&self) -> Vec<u8> {
        let a = bincode::serialize(&self.specialization_constant_data).unwrap();

        return a;
    }

    pub fn create_pipeline_layout(&self, device: &VkLogicalDevice) -> PipelineLayout {
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
