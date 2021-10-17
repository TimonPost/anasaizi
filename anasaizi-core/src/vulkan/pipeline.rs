use crate::{
    utils::any_as_u8_slice,
    vulkan,
    vulkan::{ShaderSet, VkLogicalDevice, VkRenderPass, VkSwapChain},
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{CommandBuffer, Extent2D},
};

use std::{ffi::CString, ops::Deref};

pub struct VkPipelineBuilder {
    p_viewport_state: vk::PipelineViewportStateCreateInfo,
    p_rasterization_state: vk::PipelineRasterizationStateCreateInfo,
    p_depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo,
    p_input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo,
    p_multisample_state: vk::PipelineMultisampleStateCreateInfo,
    p_dynamic_state: vk::PipelineDynamicStateCreateInfo,
    color_blend_attachment_states: Vec<vk::PipelineColorBlendAttachmentState>,
    p_color_blend_state: vk::PipelineColorBlendStateCreateInfo,
    layout: vk::PipelineLayout,
    vertex_input_state: vk::PipelineVertexInputStateCreateInfo,
    renderpass: VkRenderPass,
    subpass: u32,

    // used for pointers
    dynamic_states: Vec<vk::DynamicState>,
    viewports: Vec<vk::Viewport>,
    scissors: Vec<vk::Rect2D>,
}

impl VkPipelineBuilder {
    fn new() -> VkPipelineBuilder {
        VkPipelineBuilder {
            p_viewport_state: Default::default(),
            p_rasterization_state: Default::default(),
            p_depth_stencil_state: Default::default(),
            p_input_assembly_state: Default::default(),
            p_multisample_state: Default::default(),
            p_dynamic_state: Default::default(),

            color_blend_attachment_states: vec![],
            p_color_blend_state: Default::default(),
            layout: Default::default(),
            vertex_input_state: Default::default(),
            renderpass: unsafe { std::mem::zeroed() },
            subpass: 0,

            dynamic_states: vec![],
            viewports: vec![],
            scissors: vec![],
        }
    }

    fn with_viewport(mut self, swap_chain_extend: Extent2D) -> VkPipelineBuilder {
        self.viewports = vec![vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swap_chain_extend.width as f32,
            height: swap_chain_extend.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        self.scissors = vec![vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swap_chain_extend,
        }];

        self.p_viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&self.scissors)
            .viewports(&self.viewports)
            .build();

        self
    }

    fn with_rasterization_info(mut self) -> VkPipelineBuilder {
        self.p_rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .build();

        self
    }

    fn with_depth_stage(mut self) -> VkPipelineBuilder {
        let stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .build();

        self.p_depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_compare_op(vk::CompareOp::LESS)
            .stencil_test_enable(true) // can be enabled
            .front(stencil_state)
            .back(stencil_state)
            .depth_bounds_test_enable(false) // can be enabled
            .max_depth_bounds(1.0)
            .min_depth_bounds(0.0)
            .depth_test_enable(true)
            .depth_write_enable(true)
            .build();

        self
    }

    fn with_input_assembly_state(mut self) -> VkPipelineBuilder {
        self.p_input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();

        self
    }

    fn with_multilsample(mut self) -> VkPipelineBuilder {
        self.p_multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false)
            .build();

        self
    }

    fn with_dynamic_state(mut self) -> VkPipelineBuilder {
        self.dynamic_states = vec![vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT];
        self.p_dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&self.dynamic_states)
            .build();

        self
    }

    fn color_blend(mut self) -> VkPipelineBuilder {
        self.color_blend_attachment_states = vec![vk::PipelineColorBlendAttachmentState::builder()
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .blend_enable(true)
            .alpha_blend_op(vk::BlendOp::SUBTRACT)
            .color_write_mask(vk::ColorComponentFlags::all())
            .build()];

        self.p_color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::COPY)
            .attachments(&self.color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        self
    }

    fn layout(mut self, pipeline_layout: vk::PipelineLayout) -> VkPipelineBuilder {
        self.layout = pipeline_layout;
        self
    }

    fn with_renderpass(mut self, renderpass: VkRenderPass) -> VkPipelineBuilder {
        self.renderpass = renderpass;
        self
    }

    fn vertex_input_state(
        mut self,
        vertex_input_state: vk::PipelineVertexInputStateCreateInfo,
    ) -> VkPipelineBuilder {
        self.vertex_input_state = vertex_input_state;
        self
    }

    fn subpass(mut self, subpass: u32) -> VkPipelineBuilder {
        self.subpass = subpass;
        self
    }

    fn build(self, device: &VkLogicalDevice, shader_set: &mut ShaderSet) -> vk::Pipeline {
        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let data = shader_set.io.specialization_data_ref();
        let entries = shader_set.io.specialization_constants.clone();

        let specialisation_info = if shader_set.io.specialization_constants.len() != 0 {
            vk::SpecializationInfo::builder()
                .map_entries(&entries)
                .data(&data)
                .build()
        } else {
            unsafe { std::mem::zeroed() }
        };

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_set.vertex_shader())
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::VERTEX)
                .specialization_info(&specialisation_info)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_set.fragment_shader())
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .specialization_info(&specialisation_info)
                .build(),
        ];

        let pipeline_info = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&self.vertex_input_state)
            .input_assembly_state(&self.p_input_assembly_state)
            .viewport_state(&self.p_viewport_state)
            .rasterization_state(&self.p_rasterization_state)
            .multisample_state(&self.p_multisample_state)
            .depth_stencil_state(&self.p_depth_stencil_state)
            .color_blend_state(&self.p_color_blend_state)
            .layout(self.layout)
            .base_pipeline_index(-1)
            .subpass(self.subpass)
            .render_pass(*self.renderpass)
            .dynamic_state(&self.p_dynamic_state)
            .build()];

        let pipeline = unsafe {
            device
                .logical_device()
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_info, None)
                .map_err(|e| e.1)
                .unwrap()[0]
        };

        pipeline
    }
}

/// A Vulkan Pipeline.
///
/// A pipeline is a definition of how the GPU processes a vertices and textures all the way to the pixels in the render targets.
pub struct VkPipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    pub shader: ShaderSet,
    pipeline_id: u32,
}

impl VkPipeline {
    /// Creates a new `Pipeline`.
    pub fn create(
        device: &VkLogicalDevice,
        swapchain_extent: vk::Extent2D,
        render_pass: &VkRenderPass,
        mut shader_set: ShaderSet,
        pipeline_id: u32,
    ) -> VkPipeline {
        let builder =
            Self::pipeline_builder(device, swapchain_extent, render_pass, &mut shader_set);

        let layout = builder.layout;
        let pipeline = builder.build(device, &mut shader_set);

        VkPipeline {
            layout: layout,
            pipeline: pipeline,
            shader: shader_set,
            pipeline_id,
        }
    }

    fn pipeline_builder(
        device: &VkLogicalDevice,
        swapchain_extent: vk::Extent2D,
        render_pass: &VkRenderPass,
        shader_set: &mut ShaderSet,
    ) -> VkPipelineBuilder {
        let pipeline_layout = shader_set.io.create_pipeline_layout(device);

        VkPipelineBuilder::new()
            .vertex_input_state(shader_set.io.vertex_input_info())
            .with_input_assembly_state()
            .with_viewport(swapchain_extent)
            .with_rasterization_info()
            .with_multilsample()
            .with_depth_stage()
            .color_blend()
            .layout(pipeline_layout)
            .subpass(0)
            .with_renderpass(render_pass.clone())
            .with_dynamic_state()
    }

    pub fn ui_pipeline(
        device: &VkLogicalDevice,
        render_pass: &VkRenderPass,
        mut shader_set: ShaderSet,
        pipeline_id: u32,
    ) -> VkPipeline {
        let pipeline_layout = shader_set.io.create_pipeline_layout(device);

        let pipeline = VkPipelineBuilder::new()
            .with_rasterization_info()
            .with_multilsample()
            .with_input_assembly_state()
            .color_blend()
            .with_viewport(Default::default())
            .with_dynamic_state()
            .with_input_assembly_state()
            .layout(pipeline_layout)
            .vertex_input_state(shader_set.io.vertex_input_info())
            .with_renderpass(render_pass.clone())
            .subpass(1)
            .build(device, &mut shader_set);

        VkPipeline {
            layout: pipeline_layout,
            pipeline,
            shader: shader_set,
            pipeline_id,
        }
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    /// Refreshes the pipeline.
    /// This will destroy the pipeline and layout but will keep the shaders and meshes.
    pub unsafe fn refresh(
        &mut self,
        device: &VkLogicalDevice,
        swapchain: &VkSwapChain,
        render_pass: &VkRenderPass,
    ) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.layout, None);

        let builder =
            Self::pipeline_builder(device, swapchain.extent, render_pass, &mut self.shader);

        self.layout = builder.layout;
        self.pipeline = builder.build(device, &mut self.shader);
    }

    /// Destroys the pipeline and its contents:
    /// - Pipeline
    /// - Pipeline layout
    /// - Shaders
    /// - Meshes
    pub unsafe fn destroy(&self, device: &VkLogicalDevice) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.layout, None);

        self.shader.destroy(device);
    }

    pub fn pipeline_id(&self) -> u32 {
        self.pipeline_id
    }

    pub fn push_constants<T: Sized + Copy>(
        &self,
        device: &ash::Device,
        command_buffer: &CommandBuffer,
        pipeline: &vulkan::VkPipeline,
        data: T,
    ) {
        unsafe {
            let push_constants1 = any_as_u8_slice(&data);

            device.cmd_push_constants(
                *command_buffer,
                pipeline.layout(),
                self.shader.io.push_constant_ranges[0].stage_flags,
                0,
                &push_constants1,
            );
        }
    }
}

impl Deref for VkPipeline {
    type Target = vk::Pipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}
