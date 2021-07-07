use crate::vulkan::{LogicalDevice, RenderPass};
use ash::{version::DeviceV1_0, vk, vk::ImageView};
use std::{ops::Deref, ptr};

/// A Vulkan Framebuffer.
///
/// Framebuffers represent a collection of memory attachments that are used by a render pass instance.
/// A framebuffer provides the attachments that a render pass needs while rendering.
pub struct FrameBuffer {
    frame_buffer: vk::Framebuffer,
}

impl FrameBuffer {
    pub fn create(
        device: &LogicalDevice,
        render_pass: &RenderPass,
        image_view: ImageView,
        swapchain_extent: &vk::Extent2D,
    ) -> FrameBuffer {
        let attachments = [image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass: *render_pass.deref(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: swapchain_extent.width,
            height: swapchain_extent.height,
            layers: 1,
        };

        let frame_buffer = unsafe {
            device
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create Framebuffer!")
        };

        FrameBuffer { frame_buffer }
    }
}

impl Deref for FrameBuffer {
    type Target = vk::Framebuffer;

    fn deref(&self) -> &Self::Target {
        &self.frame_buffer
    }
}

pub struct FrameBuffers {
    frame_buffers: Vec<FrameBuffer>,
}

impl FrameBuffers {
    pub fn create(
        device: &LogicalDevice,
        render_pass: &RenderPass,
        image_views: &Vec<vk::ImageView>,
        swapchain_extent: &vk::Extent2D,
    ) -> FrameBuffers {
        let mut frame_buffers = vec![];

        for &image_view in image_views.iter() {
            frame_buffers.push(FrameBuffer::create(
                device,
                render_pass,
                image_view,
                swapchain_extent,
            ));
        }

        FrameBuffers { frame_buffers }
    }

    pub fn len(&self) -> usize {
        self.frame_buffers.len()
    }

    pub fn get(&self, index: usize) -> vk::Framebuffer {
        *self.frame_buffers[index]
    }
}
