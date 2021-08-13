use crate::vulkan::LogicalDevice;
use ash::{version::DeviceV1_0, vk};
use std::ops::Deref;

/// Vulkan Image View.
///
/// An image view is a view into an image.
/// It describes how to access the image and which part of the image to access.
#[derive(Clone)]
pub struct ImageView {
    image_view: vk::ImageView,
}

impl ImageView {
    pub fn create(
        device: &ash::Device,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
    ) -> ImageView {
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: aspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(image)
            .build();

        let image_view = unsafe {
            device
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View!")
        };

        ImageView { image_view }
    }

    pub(crate) unsafe fn destroy(&self, device: &LogicalDevice) {
        device.destroy_image_view(self.image_view, None);
    }
}

impl Deref for ImageView {
    type Target = vk::ImageView;

    fn deref(&self) -> &Self::Target {
        &self.image_view
    }
}
