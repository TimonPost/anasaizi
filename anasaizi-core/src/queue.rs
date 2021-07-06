use crate::structures::QueueFamilyIndices;
use crate::LogicalDevice;
use ash::version::DeviceV1_0;
use ash::vk;
use std::fmt;
use std::ops::Deref;

pub struct Queue {
    queue: vk::Queue,
}

impl Queue {
    pub fn create(device: &LogicalDevice, queue_index: u32) -> Queue {
        let queue = unsafe { device.get_device_queue(queue_index, 0) };

        Queue { queue }
    }
}

impl Deref for Queue {
    type Target = vk::Queue;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}


pub struct QueueFamilyProperties {
    pub queue_flags: vk::QueueFlags,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    pub min_image_transfer_granularity: vk::Extent3D,
}

impl QueueFamilyProperties {
    pub(crate) fn is_graphics(&self) -> bool {
        self.queue_flags.contains(vk::QueueFlags::GRAPHICS)
    }
}

impl From<vk::QueueFamilyProperties> for QueueFamilyProperties {
    fn from(queue: vk::QueueFamilyProperties) -> Self {
        QueueFamilyProperties {
            queue_count: queue.queue_count,
            queue_flags: queue.queue_flags,
            timestamp_valid_bits: queue.timestamp_valid_bits,
            min_image_transfer_granularity: queue.min_image_transfer_granularity,
        }
    }
}

impl fmt::Debug for QueueFamilyProperties {
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