use crate::{
    model::Mesh,
    utils::any_as_u8_slice,
    vulkan::{
        CommandPool, FrameBuffers, LogicalDevice, MeshPushConstants, Pipeline, RenderPass,
        UniformBufferObjectTemplate,
    },
};
use ash::{version::DeviceV1_0, vk};
use std::ops::Deref;

/// A Vulkan command buffer.
///
/// Commands in Vulkan, like drawing operations and memory transfers, are not executed directly using function calls.
/// You have to record all of the operations you want to perform in command buffer objects.
/// The advantage of this is that all of the hard work of setting up the drawing commands can be done in advance and in multiple threads.
/// After that, you just have to tell Vulkan to execute the commands in the main loop.
pub struct CommandBuffers {
    command_buffers: Vec<vk::CommandBuffer>,
    active_buffer: usize,
}

impl CommandBuffers {
    /// Begins the render session.
    ///
    /// 1. Begins the renderpass.
    /// 2. Begins the commandbuffer recording.
    pub fn begin_session(
        &mut self,
        device: &LogicalDevice,
        render_pass: &RenderPass,
        surface_extent: vk::Extent2D,
        framebuffers: &FrameBuffers,
        index: usize,
    ) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
        self.active_buffer = index;

        let command_buffer = self.current();

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_extent,
            })
            .framebuffer(framebuffers.get(index))
            .clear_values(&clear_values)
            .render_pass(**render_pass)
            .build();

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    /// Binds a pipeline to the current render session.
    pub fn bind_pipeline<U: UniformBufferObjectTemplate>(
        &self,
        device: &LogicalDevice,
        pipeline: &Pipeline<U>,
    ) {
        let command_buffer = self.current();

        unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, **pipeline)
        };
    }

    /// Records draw commands, required for this mesh, into the current activated command buffer.
    pub fn render_mesh<U: UniformBufferObjectTemplate>(
        &self,
        device: &LogicalDevice,
        pipeline: &Pipeline<U>,
        mesh: &Mesh,
    ) {
        let vertex_buffer = *mesh.vertex_buffer().deref();
        let index_buffer = *mesh.index_buffer().deref();
        let command_buffer = self.current();

        // Push the model matrix using push constants.
        unsafe {
            let transform = MeshPushConstants {
                model_matrix: mesh.model_transform(),
                texture_id: mesh.texture_id,
            };
            let push = any_as_u8_slice(&transform);
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout(),
                vk::ShaderStageFlags::VERTEX,
                0,
                push,
            )
        };

        let offsets = [0];
        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &offsets);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);

            let sets = pipeline
                .shader
                .get_descriptor_sets(self.active_buffer, String::from("viking"));
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout(),
                0,
                &sets,
                &[],
            );

            device.cmd_draw_indexed(command_buffer, mesh.indices_count() as u32, 1, 0, 0, 0);
        }
    }

    /// Ends the render session.
    ///
    /// 1. Ends the renderpass.
    /// 2. Ends the commandbuffer recording.
    pub fn end_session(&self, device: &LogicalDevice) {
        let command_buffer = self.current();

        unsafe {
            device.cmd_end_render_pass(command_buffer);
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    pub fn current(&self) -> vk::CommandBuffer {
        self.command_buffers[self.active_buffer]
    }

    pub fn create<U: UniformBufferObjectTemplate>(
        device: &LogicalDevice,
        command_pool: &CommandPool,
        framebuffers_count: usize,
    ) -> CommandBuffers {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(**command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(framebuffers_count as u32)
            .build();

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        };

        CommandBuffers {
            command_buffers: command_buffers,
            active_buffer: 0,
        }
    }

    /// Gets an allocated command buffer for the given index.
    pub fn get(&self, index: usize) -> vk::CommandBuffer {
        self.command_buffers[index]
    }

    /// Frees the command buffer memory.
    pub unsafe fn free(&self, device: &LogicalDevice, command_pool: &CommandPool) {
        device.free_command_buffers(**command_pool, &self.command_buffers)
    }
}
