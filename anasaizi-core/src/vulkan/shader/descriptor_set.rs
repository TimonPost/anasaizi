use ash::{version::DeviceV1_0, vk};
use std::ops::Deref;

/// Think of a single descriptor as a handle or pointer into a resource.
/// That resource being a Buffer or a Image, and also holds other information, such as the size of the buffer, or the type of sampler if it’s for an image.
/// A VkDescriptorSet is a pack of those pointers that are bound together.
pub struct DescriptorSet {
    descriptor_set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub fn new(
        device: &ash::Device,
        descriptor_set: vk::DescriptorSet,
        descriptor_write_sets: Vec<vk::WriteDescriptorSet>,
    ) -> DescriptorSet {
        let mut descriptor_write_sets = descriptor_write_sets;
        for descriptor_write_set in descriptor_write_sets.iter_mut() {
            descriptor_write_set.dst_set = descriptor_set;
        }
        unsafe {
            device.update_descriptor_sets(&descriptor_write_sets, &[]);
        }

        DescriptorSet { descriptor_set }
    }
}

impl Deref for DescriptorSet {
    type Target = vk::DescriptorSet;

    fn deref(&self) -> &Self::Target {
        &self.descriptor_set
    }
}
