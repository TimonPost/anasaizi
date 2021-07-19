use crate::{
    math::Vertex,
    vulkan::{CommandPool, Instance, LogicalDevice, Queue},
};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
    vk::{Buffer, DeviceMemory, StructureType},
};
use std::{mem::size_of, ops::Deref, ptr};

pub fn create_buffer(
    instance: &Instance,
    device: &LogicalDevice,
    size: u64,
    usage: vk::BufferUsageFlags,
    flags: vk::MemoryPropertyFlags,
) -> (Buffer, DeviceMemory) {
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

pub fn copy_buffer(
    device: &LogicalDevice,
    submit_queue: &Queue,
    command_pool: &CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(1)
        .command_pool(**command_pool)
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
            .expect("Failed to begin Command Buffer");

        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];

        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);

        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end Command Buffer");
    }

    let submit_info = [vk::SubmitInfo::builder()
        .command_buffers(&command_buffers)
        .build()];

    unsafe {
        device
            .queue_submit(**submit_queue, &submit_info, vk::Fence::null())
            .expect("Failed to Submit Queue.");
        device
            .queue_wait_idle(**submit_queue)
            .expect("Failed to wait Queue idle");

        device.free_command_buffers(**command_pool, &command_buffers);
    }
}
