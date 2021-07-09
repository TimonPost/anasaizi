use crate::vulkan::{CommandPool, FrameBuffers, LogicalDevice, Pipeline, RenderPass, VertexBuffer, Instance};
use ash::{version::DeviceV1_0, vk};
use std::ptr;
use crate::model::{triangle_vertices, Mesh};
use std::ops::Deref;

/// A Vulkan command buffer.
///
/// Commands in Vulkan, like drawing operations and memory transfers, are not executed directly using function calls.
/// You have to record all of the operations you want to perform in command buffer objects.
/// The advantage of this is that all of the hard work of setting up the drawing commands can be done in advance and in multiple threads.
/// After that, you just have to tell Vulkan to execute the commands in the main loop.
pub struct CommandBuffers {
    command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandBuffers {
    pub fn create(
        device: &LogicalDevice,
        command_pool: &CommandPool,
        graphics_pipeline: &Pipeline,
        framebuffers: &FrameBuffers,
        render_pass: &RenderPass,
        surface_extent: vk::Extent2D,
        mesh: &Mesh
    ) -> CommandBuffers {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: framebuffers.len() as u32,
            command_pool: **command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next: ptr::null(),
                p_inheritance_info: ptr::null(),
                flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
            };

            unsafe {
                device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Failed to begin recording Command Buffer at beginning!");
            }

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass: **render_pass,
                framebuffer: framebuffers.get(i),
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: surface_extent,
                },
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };

            let offsets = [0 as u64];

            unsafe {
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    **graphics_pipeline,
                );

                let vertex_buffer = *mesh.vertex_buffer().deref();
                let index_buffer = *mesh.index_buffer().deref();

                device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer] , &offsets);
                device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT16);

                device.cmd_draw_indexed(command_buffer, mesh.indices_count() as u32, 1, 0, 0, 0);

                device.cmd_end_render_pass(command_buffer);

                device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        CommandBuffers { command_buffers }
    }

    pub fn get(&self, index: usize) -> vk::CommandBuffer {
        self.command_buffers[index]
    }
}
