use crate::{
    engine::RenderObject,
    vulkan::{
        CommandPool, FrameBuffers, LogicalDevice, RenderPass,
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
}

impl CommandBuffers {
    pub fn create<U: UniformBufferObjectTemplate>(
        device: &LogicalDevice,
        command_pool: &CommandPool,
        render_objects: &Vec<RenderObject<U>>,
        framebuffers: &FrameBuffers,
        render_pass: &RenderPass,
        surface_extent: vk::Extent2D,
    ) -> CommandBuffers {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(**command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(framebuffers.len() as u32)
            .build();

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

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
                .framebuffer(framebuffers.get(i))
                .clear_values(&clear_values)
                .render_pass(**render_pass)
                .build();

            let offsets = [0 as u64];

            unsafe {
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
            }

            for render_object in render_objects {
                unsafe {
                    device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        *render_object.pipeline,
                    );

                    let vertex_buffer = *render_object.mesh.vertex_buffer().deref();
                    let index_buffer = *render_object.mesh.index_buffer().deref();

                    device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &offsets);
                    device.cmd_bind_index_buffer(
                        command_buffer,
                        index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );

                    let sets = [*render_object.shader.descriptor_sets[i]];
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        render_object.pipeline.layout(),
                        0,
                        &sets,
                        &[],
                    );

                    device.cmd_draw_indexed(
                        command_buffer,
                        render_object.mesh.indices_count() as u32,
                        1,
                        0,
                        0,
                        0,
                    );
                }
            }

            unsafe {
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
