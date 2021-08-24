mod buffer;
mod command_buffer;
mod framebuffer;
mod index_buffer;
mod uniform_buffer;
mod vertex_buffer;

pub use command_buffer::CommandBuffers;
pub use framebuffer::{FrameBuffer, FrameBuffers};
pub use index_buffer::IndexBuffer;
pub use uniform_buffer::UniformBuffer;
pub use vertex_buffer::VertexBuffer;

use crate::{
    engine::RenderContext,
    vulkan::{CommandPool, LogicalDevice, Queue},
};
use ash::{version::DeviceV1_0, vk};
pub use buffer::{copy_image_to_buffer, create_allocate_vk_buffer};
use std::ptr;

/// Creates a commandbuffer from the given pool which can be used to executed command on.
pub fn begin_single_time_command(render_context: &RenderContext) -> vk::CommandBuffer {
    let device = render_context.device();

    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(1)
        .command_pool(render_context.command_pool())
        .level(vk::CommandBufferLevel::PRIMARY)
        .build();

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&allocate_info)
            .expect("Failed to allocate Command Buffer")
    };

    let command_buffer = command_buffers[0];

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .build();

    unsafe {
        device
            .begin_command_buffer(command_buffer, &begin_info)
            .expect("Failed to begin recording Command Buffer at beginning!");
    }

    command_buffer
}

/// Finalizes the given commandbuffer and submits it to the given queue
pub fn end_single_time_command(render_context: &RenderContext, command_buffer: &vk::CommandBuffer) {
    let device = render_context.device();

    unsafe {
        device
            .end_command_buffer(*command_buffer)
            .expect("Failed to record Command Buffer at Ending!");
    }

    let buffers_to_submit = [*command_buffer];

    let submit_infos = [vk::SubmitInfo {
        s_type: vk::StructureType::SUBMIT_INFO,
        p_next: ptr::null(),
        wait_semaphore_count: 0,
        p_wait_semaphores: ptr::null(),
        p_wait_dst_stage_mask: ptr::null(),
        command_buffer_count: 1,
        p_command_buffers: buffers_to_submit.as_ptr(),
        signal_semaphore_count: 0,
        p_signal_semaphores: ptr::null(),
    }];

    unsafe {
        device
            .queue_submit(
                render_context.graphics_queue(),
                &submit_infos,
                vk::Fence::null(),
            )
            .expect("Failed to Queue Submit!");
        device
            .queue_wait_idle(render_context.graphics_queue())
            .expect("Failed to wait Queue idle!");
        device.free_command_buffers(render_context.command_pool(), &buffers_to_submit);
    }
}
