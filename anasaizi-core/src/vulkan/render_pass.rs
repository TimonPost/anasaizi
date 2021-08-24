use crate::vulkan::{Instance, LogicalDevice};
use ash::{version::DeviceV1_0, vk};
use std::{mem::zeroed, ops::Deref};

pub struct SubpassDescriptor {
    color: (bool, usize),
    depth: (bool, usize),
}

impl SubpassDescriptor {
    pub fn new() -> SubpassDescriptor {
        SubpassDescriptor {
            color: (false, usize::MAX),
            depth: (false, usize::MAX),
        }
    }
    pub fn with_color(mut self, attachment_index: usize) -> SubpassDescriptor {
        self.color = (true, attachment_index);
        self
    }

    pub fn with_depth(mut self, attachment_index: usize) -> SubpassDescriptor {
        self.depth = (true, attachment_index);
        self
    }
}

pub struct RenderPassBuilder {
    attachments: Vec<vk::AttachmentDescription>,
    color_attachment_refs: Vec<vk::AttachmentReference>,
    depth_attachment_refs: vk::AttachmentReference,
    subpasses: Vec<vk::SubpassDescription>,
    dependencies: Vec<vk::SubpassDependency>,
}

impl RenderPassBuilder {
    pub fn builder() -> RenderPassBuilder {
        RenderPassBuilder {
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
    ) -> RenderPassBuilder {
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
    ) -> RenderPassBuilder {
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
        subpasses: Vec<SubpassDescriptor>,
        deps: &[vk::SubpassDependency],
    ) -> RenderPassBuilder {
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

    pub fn build(mut self, device: &LogicalDevice) -> RenderPass {
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

        RenderPass { render_pass }
    }
}

/// A Vulkan Renderpass.
///
/// A render pass describes the scope of a rendering operation by specifying the collection of attachments,
/// subpasses, and dependencies used during the rendering operation.
/// A render pass consists of at least one subpass.
/// The communication of this information to the driver allows the driver to know what to
/// expect when rendering begins and to set up the hardware optimally for the rendering operation.
pub struct RenderPass {
    render_pass: vk::RenderPass,
}

impl RenderPass {
    /// Destroys the render pass.
    pub unsafe fn destroy(&self, device: &LogicalDevice) {
        device.destroy_render_pass(self.render_pass, None);
    }
}

impl Deref for RenderPass {
    type Target = vk::RenderPass;

    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}
