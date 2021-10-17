use crate::vulkan::VkLogicalDevice;
use ash::{version::DeviceV1_0, vk};
use std::ops::Deref;

pub struct VkSubpassDescriptor {
    color: (bool, usize),
    depth: (bool, usize),
}

impl VkSubpassDescriptor {
    pub fn new() -> VkSubpassDescriptor {
        VkSubpassDescriptor {
            color: (false, usize::MAX),
            depth: (false, usize::MAX),
        }
    }
    pub fn with_color(mut self, attachment_index: usize) -> VkSubpassDescriptor {
        self.color = (true, attachment_index);
        self
    }

    pub fn with_depth(mut self, attachment_index: usize) -> VkSubpassDescriptor {
        self.depth = (true, attachment_index);
        self
    }
}

pub struct VkRenderPassBuilder {
    attachments: Vec<vk::AttachmentDescription>,
    color_attachment_refs: Vec<vk::AttachmentReference>,
    depth_attachment_refs: vk::AttachmentReference,
    subpasses: Vec<vk::SubpassDescription>,
    dependencies: Vec<vk::SubpassDependency>,
}

impl VkRenderPassBuilder {
    pub fn builder() -> VkRenderPassBuilder {
        VkRenderPassBuilder {
            attachments: Vec::new(),
            subpasses: Vec::new(),
            color_attachment_refs: Vec::new(),
            depth_attachment_refs: vk::AttachmentReference::default(),
            dependencies: Vec::new(),
        }
    }

    pub fn add_color_attachment(
        mut self,
        attachment_number: u32,
        format: vk::Format,
        final_layout: vk::ImageLayout,
        layout: vk::ImageLayout,
    ) -> VkRenderPassBuilder {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .final_layout(final_layout)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(attachment_number)
            .layout(layout) // | vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            .build();

        self.attachments.push(color_attachment);
        self.color_attachment_refs.push(color_attachment_ref);
        self
    }

    pub fn add_depth_attachment(
        mut self,
        attachment_number: u32,
        format: vk::Format,
    ) -> VkRenderPassBuilder {
        let depth_color_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let depth_color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(attachment_number)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        self.attachments.push(depth_color_attachment);
        self.depth_attachment_refs = depth_color_attachment_ref;

        self
    }

    pub fn add_subpasses(
        mut self,
        subpasses: Vec<VkSubpassDescriptor>,
        deps: &[vk::SubpassDependency],
    ) -> VkRenderPassBuilder {
        self.dependencies.extend_from_slice(deps);

        for subpass in subpasses {
            let mut subpass_builder = vk::SubpassDescription::builder();
            if subpass.color.0 {
                subpass_builder = subpass_builder.color_attachments(&self.color_attachment_refs);
            }
            if subpass.depth.0 {
                subpass_builder =
                    subpass_builder.depth_stencil_attachment(&self.depth_attachment_refs);
            }

            self.subpasses.push(
                subpass_builder
                    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                    .build(),
            );
        }

        self
    }

    pub fn build(self, device: &VkLogicalDevice) -> VkRenderPass {
        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&self.attachments)
            .subpasses(&self.subpasses)
            .dependencies(&self.dependencies)
            .build();

        let render_pass = unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        };

        VkRenderPass { render_pass }
    }
}

/// A Vulkan Renderpass.
///
/// A render pass describes the scope of a rendering operation by specifying the collection of attachments,
/// subpasses, and dependencies used during the rendering operation.
/// A render pass consists of at least one subpass.
/// The communication of this information to the driver allows the driver to know what to
/// expect when rendering begins and to set up the hardware optimally for the rendering operation.
#[derive(Clone)]
pub struct VkRenderPass {
    render_pass: vk::RenderPass,
}

impl VkRenderPass {
    /// Destroys the render pass.
    pub unsafe fn destroy(&self, device: &VkLogicalDevice) {
        device.destroy_render_pass(self.render_pass, None);
    }
}

impl Deref for VkRenderPass {
    type Target = vk::RenderPass;

    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}
