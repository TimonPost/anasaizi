use crate::{
    engine::RenderContext,
    vulkan::{
        begin_single_time_command, end_single_time_command, CommandPool, Instance, LogicalDevice,
        Queue,
    },
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{Buffer, DeviceMemory, Extent2D, Extent3D, Offset3D},
};

/// Creates and allocates a vulkan buffer.
///
/// # Arguments
/// - `size`: The size in bytes of the buffer that is to be created.
/// - `usage`: How the buffer will be used.
/// - `flags`: What the memory properties of this buffer should be.
pub fn create_allocate_vk_buffer(
    render_context: &RenderContext,
    size: u64,
    usage: vk::BufferUsageFlags,
    flags: vk::MemoryPropertyFlags,
) -> (Buffer, DeviceMemory) {
    let device = render_context.device();

    // Create buffer.
    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .build();

    let buffer = unsafe {
        device
            .create_buffer(&buffer_create_info, None)
            .expect("Could not create vertex buffer.")
    };

    // Allocate buffer.
    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_type = render_context.find_memory_type(mem_requirements.memory_type_bits, flags);

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type)
        .build();

    let buffer_memory = unsafe {
        device
            .allocate_memory(&allocate_info, None)
            .expect("Could not allocate memory for buffer image.")
    };

    unsafe {
        device
            .bind_buffer_memory(buffer, buffer_memory, 0)
            .expect("Failed to bind Buffer");
    }

    (buffer, buffer_memory)
}

/// Copy one buffer over to an other buffer.
///
/// # Arguments
/// - `submit_queue`: The queue that will be used to execute this copy command on.
/// - `command_pool`: The pool that will be used to allocate the command buffer, used for copy operation, from.
/// - `src_buffer`: The source buffer that will be copied to the `dst_buffer`.
/// - `dst_buffer`: The destination buffer into which the data from `src_buffer` will be copied.
/// - `size`: The size of data that will be copied. Offset is 0.
pub fn copy_buffer(
    render_context: &RenderContext,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let command_buffer = begin_single_time_command(render_context);

    unsafe {
        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];

        render_context.device().cmd_copy_buffer(
            command_buffer,
            src_buffer,
            dst_buffer,
            &copy_regions,
        );
    }

    end_single_time_command(render_context, &command_buffer);
}

/// Copy one buffer over to an other buffer.
///
/// # Arguments
/// - `submit_queue`: The queue that will be used to execute this copy command on.
/// - `command_pool`: The pool that will be used to allocate the command buffer, used for copy operation, from.
/// - `src_buffer`: The source buffer that will be copied to the `dst_buffer`.
/// - `dst_buffer`: The destination buffer into which the data from `src_buffer` will be copied.
/// - `size`: The size of data that will be copied. Offset is 0.
pub fn copy_image_to_buffer(
    render_context: &RenderContext,
    src_image: vk::Image,
    src_image_layout: vk::ImageLayout,
    dst_buffer: vk::Buffer,
    command_buffer: vk::CommandBuffer,
    size: u64,
    image_extent: Extent2D,
) {
    unsafe {
        let copy_regions = [vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: image_extent.width as u32,
            buffer_image_height: image_extent.height as u32,
            image_extent: Extent3D::builder()
                .width(image_extent.width)
                .height(image_extent.height)
                .depth(1)
                .build(),
            image_offset: Offset3D { x: 0, y: 0, z: 0 },
            image_subresource: vk::ImageSubresourceLayers::builder()
                .layer_count(1)
                .mip_level(0)
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .build(),
        }];

        render_context.device().cmd_copy_image_to_buffer(
            command_buffer,
            src_image,
            src_image_layout,
            dst_buffer,
            &copy_regions,
        );
    }
}
