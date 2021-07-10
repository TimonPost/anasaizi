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
    let buffer_create_info = vk::BufferCreateInfo {
        s_type: StructureType::BUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
    };

    let buffer = unsafe {
        device
            .create_buffer(&buffer_create_info, None)
            .expect("Could not create vertex buffer.")
    };

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let mem_properties =
        unsafe { instance.get_physical_device_memory_properties(*device.physical_device()) };
    let memory_type =
        device.find_memory_type(mem_requirements.memory_type_bits, flags, mem_properties);

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: ptr::null(),
        allocation_size: mem_requirements.size,
        memory_type_index: memory_type,
    };

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
    let allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: 1,
        command_pool: **command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&allocate_info)
            .expect("Failed to allocate Command Buffer")
    };

    let command_buffer = command_buffers[0];

    let begin_info = vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        p_next: ptr::null(),
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        p_inheritance_info: ptr::null(),
    };

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

    let submit_info = [vk::SubmitInfo {
        s_type: vk::StructureType::SUBMIT_INFO,
        p_next: ptr::null(),
        wait_semaphore_count: 0,
        p_wait_semaphores: ptr::null(),
        p_wait_dst_stage_mask: ptr::null(),
        command_buffer_count: 1,
        p_command_buffers: &command_buffer,
        signal_semaphore_count: 0,
        p_signal_semaphores: ptr::null(),
    }];

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
