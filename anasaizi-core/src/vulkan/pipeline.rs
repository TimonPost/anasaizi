use crate::vulkan::{LogicalDevice, RenderPass, ShaderSet, SwapChain};
use ash::{version::DeviceV1_0, vk, vk::PipelineLayout};
use std::{ffi::CString, mem, ops::Deref, ptr};

/// A Vulkan Pipeline.
///
/// A pipeline is a definition of how the GPU processes a vertices and textures all the way to the pixels in the render targets.
pub struct Pipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    pub shader: ShaderSet,
    pipeline_id: u32,
}

impl Pipeline {
    /// Creates a new `Pipeline`.
    pub fn create(
        device: &LogicalDevice,
        swapchain_extent: vk::Extent2D,
        render_pass: &RenderPass,
        mut shader_set: ShaderSet,
        pipeline_id: u32,
    ) -> Pipeline {
        let (pipeline, layout) = Self::build_pipeline(
            device,
            swapchain_extent,
            render_pass,
            &mut shader_set,
        );

        Pipeline {
            layout,
            pipeline,
            shader: shader_set,
            pipeline_id,
        }
    }

    fn build_pipeline(
        device: &LogicalDevice,
        swapchain_extent: vk::Extent2D,
        render_pass: &RenderPass,
        shader_set: &mut ShaderSet,
    ) -> (ash::vk::Pipeline, PipelineLayout) {
        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_set.vertex_shader())
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_set.fragment_shader())
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        let pipeline_layout = shader_set.io.create_pipeline_layout(device);
        let vertex_input_info = shader_set.io.vertex_input_info();

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        }];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports)
            .build();

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: vk::FALSE,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            rasterizer_discard_enable: vk::FALSE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: vk::FALSE,
            depth_bias_slope_factor: 0.0,
        };

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false)
            .build();

        let stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .build();

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
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

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .blend_enable(true)
            .alpha_blend_op(vk::BlendOp::SUBTRACT)
            .color_write_mask(vk::ColorComponentFlags::all())
            .build()];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let dynamic_states = [vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT];
        let dynamic_states_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states)
            .build();

        let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterization_statue_create_info)
            .multisample_state(&multisample_state_create_info)
            .depth_stencil_state(&depth_state_create_info)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .base_pipeline_index(-1)
            .subpass(0)
            .render_pass(**render_pass)
            .dynamic_state(&dynamic_states_info)
            .build()];

        let graphics_pipelines = unsafe {
            device
                .logical_device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphic_pipeline_create_infos,
                    None,
                )
                .expect("Failed to create Graphics Pipeline!.")
        };

        (graphics_pipelines[0], pipeline_layout)
    }

    pub fn object_pick_pipeline(
        device: &LogicalDevice,
        render_pass: &RenderPass,
        mut shader_set: ShaderSet,
        extend: &vk::Extent2D,
    ) -> Pipeline {
        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_set.vertex_shader())
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_set.fragment_shader())
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        let pipeline_layout = shader_set.io.create_pipeline_layout(device);
        let vertex_input_info = shader_set.io.vertex_input_info();

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extend.width as f32,
            height: extend.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: *extend,
        }];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports)
            .build();

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: vk::FALSE,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            rasterizer_discard_enable: vk::FALSE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: vk::FALSE,
            depth_bias_slope_factor: 0.0,
        };

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false)
            .build();

        let stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .build();

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
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

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .blend_enable(true)
            .alpha_blend_op(vk::BlendOp::SUBTRACT)
            .color_write_mask(vk::ColorComponentFlags::all())
            .build()];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let dynamic_states = [vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT];
        let dynamic_states_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states)
            .build();

        let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterization_statue_create_info)
            .multisample_state(&multisample_state_create_info)
            .depth_stencil_state(&depth_state_create_info)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .base_pipeline_index(-1)
            .subpass(0)
            .render_pass(**render_pass)
            .dynamic_state(&dynamic_states_info)
            .build()];

        let graphics_pipelines = unsafe {
            device
                .logical_device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphic_pipeline_create_infos,
                    None,
                )
                .expect("Failed to create Graphics Pipeline!.")
        };

        Pipeline {
            layout: pipeline_layout,
            pipeline: graphics_pipelines[0],
            shader: shader_set,
            pipeline_id: 100,
        }
    }

    pub fn ui_pipeline(
        device: &LogicalDevice,
        render_pass: &RenderPass,
        mut shader_set: ShaderSet,
        pipeline_id: u32,
    ) -> Pipeline {
        let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

        let shader_states_infos = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_set.vertex_shader())
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_set.fragment_shader())
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        let pipeline_layout = shader_set.io.create_pipeline_layout(device);
        let vertex_input_info = shader_set.io.vertex_input_info();

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0);

        let viewports = [Default::default()];
        let scissors = [Default::default()];
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()];
        let color_blending_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let dynamic_states = [vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT];
        let dynamic_states_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        let pipeline_info = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_states_infos)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .rasterization_state(&rasterizer_info)
            .viewport_state(&viewport_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&color_blending_info)
            .dynamic_state(&dynamic_states_info)
            .layout(pipeline_layout)
            .render_pass(**render_pass)
            .subpass(1)
            .build()];

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_info, None)
                .map_err(|e| e.1)
                .unwrap()[0]
        };

        Pipeline {
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
        device: &LogicalDevice,
        swapchain: &SwapChain,
        render_pass: &RenderPass,
    ) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.layout, None);

        let (pipeline, layout) = Self::build_pipeline(
            device,
            swapchain.extent,
            render_pass,
            &mut self.shader,
        );
        self.pipeline = pipeline;
        self.layout = layout;
    }

    /// Destroys the pipeline and its contents:
    /// - Pipeline
    /// - Pipeline layout
    /// - Shaders
    /// - Meshes
    pub unsafe fn destroy(&self, device: &LogicalDevice) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.layout, None);

        self.shader.destroy(device);
    }

    pub fn pipeline_id(&self) -> u32 {
        self.pipeline_id
    }
}

impl Deref for Pipeline {
    type Target = vk::Pipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}
