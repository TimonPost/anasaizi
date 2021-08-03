use crate::vulkan::{
    begin_single_time_command, end_single_time_command, CommandPool, Instance, LogicalDevice, Queue,
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{Buffer, DeviceMemory},
};

/// Creates and allocates a vulkan buffer.
///
/// # Arguments
/// - `size`: The size in bytes of the buffer that is to be created.
/// - `usage`: How the buffer will be used.
/// - `flags`: What the memory properties of this buffer should be.
pub fn create_allocate_vk_buffer(
    _instance: &Instance,
    device: &LogicalDevice,
    size: u64,
    usage: vk::BufferUsageFlags,
    flags: vk::MemoryPropertyFlags,
) -> (Buffer, DeviceMemory) {
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

    let memory_type = device.find_memory_type(mem_requirements.memory_type_bits, flags);

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type)
        .build();

    let buffer_memory = unsafe {
        device
            .allocate_memory(&allocate_info, None)
            .expect("Could not allocate memory for depth buffer image.")
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
    device: &LogicalDevice,
    submit_queue: &Queue,
    command_pool: &CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let command_buffer = begin_single_time_command(device, command_pool);

    unsafe {
        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];

        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
    }

    end_single_time_command(device, command_pool, submit_queue, &command_buffer);
}
