use crate::vulkan::VkLogicalDevice;
use ash::{version::DeviceV1_0, vk};
use std::{fmt, ops::Deref};

/// A Vulkan Queue.
///
/// A queue is the abstracted mechanism used to submit commands to the hardware.
/// Vulkan arranges queues according to their type into queue families.
pub struct VkQueue {
    queue: vk::Queue,
}

impl VkQueue {
    /// Creates a a queue.
    pub fn create(device: &VkLogicalDevice, queue_index: u32) -> VkQueue {
        let queue = unsafe { device.get_device_queue(queue_index, 0) };

        VkQueue { queue }
    }
}

impl Deref for VkQueue {
    type Target = vk::Queue;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

pub struct VkQueueFamilyProperties {
    pub queue_flags: vk::QueueFlags,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    pub min_image_transfer_granularity: vk::Extent3D,
}

impl VkQueueFamilyProperties {
    pub(crate) fn is_graphics(&self) -> bool {
        self.queue_flags.contains(vk::QueueFlags::GRAPHICS)
    }
}

impl From<vk::QueueFamilyProperties> for VkQueueFamilyProperties {
    fn from(queue: vk::QueueFamilyProperties) -> Self {
        VkQueueFamilyProperties {
            queue_count: queue.queue_count,
            queue_flags: queue.queue_flags,
            timestamp_valid_bits: queue.timestamp_valid_bits,
            min_image_transfer_granularity: queue.min_image_transfer_granularity,
        }
    }
}

impl fmt::Debug for VkQueueFamilyProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        println!("\t\tQueue Count | Graphics, Compute, Transfer, Sparse Binding");

        let is_graphics_support = if self.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            "support"
        } else {
            "unsupport"
        };
        let is_compute_support = if self.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            "support"
        } else {
            "unsupport"
        };
        let is_transfer_support = if self.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            "support"
        } else {
            "unsupport"
        };
        let is_sparse_support = if self.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING) {
            "support"
        } else {
            "unsupport"
        };

        write!(
            f,
            "\t\t{}\t    | {},  {},  {},  {}",
            self.queue_count,
            is_graphics_support,
            is_compute_support,
            is_transfer_support,
            is_sparse_support
        )?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct VkQueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl VkQueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}
