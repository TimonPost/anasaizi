use crate::vulkan::{
    BufferLayout, LogicalDevice, RenderPass, ShaderSet, UniformBufferObjectTemplate,
};
use ash::{version::DeviceV1_0, vk};
use std::{ffi::CString, ops::Deref, ptr};

pub struct Pipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl Pipeline {
    // TODO: Make this pipeline more dynamic.
    pub fn create<U: UniformBufferObjectTemplate>(
        device: &LogicalDevice,
        swapchain_extent: vk::Extent2D,
        render_pass: &RenderPass,
        shader_set: &ShaderSet<U>,
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

        let attrib_descriptions = shader_set.input_buffer_layout.build_attrib_description();
        let binding_descriptions = shader_set.input_buffer_layout.build_binding_description();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&attrib_descriptions)
            .vertex_binding_descriptions(&[binding_descriptions])
            .build();

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

        let descriptor_layouts = [shader_set.descriptor_set_layout];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 1,
            p_set_layouts: descriptor_layouts.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };

        let pipeline_layout = unsafe {
            device
                .logical_device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout!")
        };

        let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_create_info)
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
        }
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Deref for Pipeline {
    type Target = vk::Pipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}
