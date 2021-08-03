use crate::vulkan::{ImageView, LogicalDevice, RenderPass};
use ash::{version::DeviceV1_0, vk};
use std::ops::Deref;

/// A Vulkan Framebuffer.
///
/// Framebuffers represent a collection of memory attachments that are used by a render pass instance.
/// A framebuffer provides the attachments that a render pass needs while rendering.
pub struct FrameBuffer {
    buffer: vk::Framebuffer,
}

impl FrameBuffer {
    pub fn create(
        device: &LogicalDevice,
        render_pass: &RenderPass,
        image_view: vk::ImageView,
        depth_image_view: ImageView,
        swapchain_extent: &vk::Extent2D,
    ) -> FrameBuffer {
        let attachments = [image_view, *depth_image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(**render_pass)
            .attachments(&attachments)
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .layers(1)
            .build();

        let frame_buffer = unsafe {
            device
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create Framebuffer!")
        };

        FrameBuffer {
            buffer: frame_buffer,
        }
    }
}

impl Deref for FrameBuffer {
    type Target = vk::Framebuffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

/// A collection of vulkan framebuffers.
pub struct FrameBuffers {
    frame_buffers: Vec<FrameBuffer>,
}

impl FrameBuffers {
    /// Creates a new collection of framebuffer for the given images.
    ///
    /// `render_pass`: The render pass that will render into the framebuffers.
    /// `image_views`: The image views defining the contents of the framebuffers.
    /// `swapchain_extent`: The dimensions of the framebufers.
    pub fn create(
        device: &LogicalDevice,
        render_pass: &RenderPass,
        image_views: &Vec<vk::ImageView>,
        depth_image_view: ImageView,
        swapchain_extent: &vk::Extent2D,
    ) -> FrameBuffers {
        let mut frame_buffers = vec![];

        for &image_view in image_views.iter() {
            frame_buffers.push(FrameBuffer::create(
                device,
                render_pass,
                image_view,
                depth_image_view.clone(),
                swapchain_extent,
            ));
        }

        FrameBuffers {
            frame_buffers: frame_buffers,
        }
    }

    /// Returns the number of framebuffers in this collection.
    pub fn len(&self) -> usize {
        self.frame_buffers.len()
    }

    /// Returns an framebuffer at the given index.
    pub fn get(&self, index: usize) -> vk::Framebuffer {
        *self.frame_buffers[index]
    }

    /// Destroys the framebuffer.
    pub(crate) unsafe fn destroy(&self, device: &LogicalDevice) {
        for framebuffer in self.frame_buffers.iter() {
            device.destroy_framebuffer(**framebuffer, None);
        }
    }
}
