use crate::vulkan::{Instance, LogicalDevice};
use ash::{version::DeviceV1_0, vk};
use std::{ops::Deref, ptr};

pub struct RenderPass {
    render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn create(
        instance: &Instance,
        device: &LogicalDevice,
        surface_format: vk::Format,
    ) -> RenderPass {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let depth_color_attachment = vk::AttachmentDescription::builder()
            .format(device.find_depth_format(&instance))
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let color_attachment_ref = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let depth_color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = [vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_ref)
            .depth_stencil_attachment(&depth_color_attachment_ref)
            .build()];

        let render_pass_attachments = [color_attachment, depth_color_attachment];

        let dependencies = [vk::SubpassDependency::builder()
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .dependency_flags(vk::DependencyFlags::BY_REGION)
            .build()];

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpass)
            .dependencies(&dependencies)
            .build();

        let render_pass = unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        };

        RenderPass { render_pass }
    }
}

impl Deref for RenderPass {
    type Target = vk::RenderPass;

    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}
