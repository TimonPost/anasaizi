use crate::{
    engine::RenderContext,
    vulkan::{
        begin_single_time_command, create_allocate_vk_buffer, end_single_time_command, VkImageView,
        VkLogicalDevice,
    },
};
use ash::{version::DeviceV1_0, vk};
use image::GenericImageView;
use std::{path::Path, ptr};

/// A vulkan texture that contains an image, imageview, and device memory.
#[derive(Clone)]
pub struct Texture {
    pub image: vk::Image,
    pub device_memory: vk::DeviceMemory,
    pub image_view: VkImageView,
}

impl Texture {
    /// Creates a texture from a path to the rgba image.
    pub fn create(render_context: &RenderContext, image_path: &Path) -> Texture {
        Self::from_path(render_context, image_path)
    }

    /// Creates a texture from a path to the rgba image.
    pub fn from_path(render_context: &RenderContext, image_path: &Path) -> Texture {
        let mut image_object = image::open(image_path).unwrap(); // this function is slow in debug mode.
        image_object = image_object.flipv();

        let (image_width, image_height) = (image_object.width(), image_object.height());

        let image_data = match &image_object {
            image::DynamicImage::ImageLuma8(_)
            | image::DynamicImage::ImageBgr8(_)
            | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba8().into_raw(),
            image::DynamicImage::ImageLumaA8(_)
            | image::DynamicImage::ImageBgra8(_)
            | image::DynamicImage::ImageRgba8(_) => image_object.to_bytes(),
            _ => {
                panic!("Invalid image format, image should be rgba compatible");
            }
        };

        Self::from_bytes(render_context, &image_data, image_width, image_height)
    }

    /// Creates a texture from an `u8` array containing an rgba image.
    pub fn from_bytes(
        render_context: &RenderContext,
        texture_data: &[u8],
        image_width: u32,
        image_height: u32,
    ) -> Texture {
        let device = render_context.device().clone();

        let image_size =
            (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;

        if image_size <= 0 {
            panic!("Failed to load texture image! Texture image is of size 0.")
        }

        // Create staging buffer on the CPU.
        // The buffer should be in host visible memory so that we can map it
        // and it should be usable as a transfer source so that we can copy it to an image later on:
        let (staging_buffer, staging_buffer_memory) = create_allocate_vk_buffer(
            render_context,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        // Copy the pixel values from the loaded image to the staging buffer.
        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    image_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(texture_data.as_ptr(), texture_data.len());

            device.unmap_memory(staging_buffer_memory);
        }

        let (texture_image, texture_image_memory) = Self::create_image(
            render_context,
            image_width,
            image_height,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        Self::transition_image_layout(
            render_context,
            texture_image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );

        Self::copy_buffer_to_image(
            render_context,
            staging_buffer,
            texture_image,
            image_width,
            image_height,
        );

        Self::transition_image_layout(
            render_context,
            texture_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );

        // Cleanup the staging buffer because the texture is now uploaded to the GPU.
        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        // Crate an imageview for this image.
        let image_view = VkImageView::create(
            render_context.device(),
            texture_image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageAspectFlags::COLOR,
        );

        Texture {
            image: texture_image,
            device_memory: texture_image_memory,
            image_view,
        }
    }

    /// Creates a vulkan image.
    ///
    /// # Arguments
    /// - `format`: The format of the texel data.
    /// - `tiling`: How texels are laid out in memory. Use OPTIMAL for best performance.
    /// - `usage`: How the image is going to be used.
    /// - `required_memory_properties`: The properties of the image memory.
    pub fn create_image(
        render_context: &RenderContext,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        required_memory_properties: vk::MemoryPropertyFlags,
    ) -> (vk::Image, vk::DeviceMemory) {
        // Crate image.
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(tiling)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .build();

        let texture_image = unsafe {
            render_context
                .device()
                .create_image(&image_create_info, None)
                .expect("Failed to create Texture Image!")
        };

        // Allocate image memory.
        let image_memory_requirement = unsafe {
            render_context
                .device()
                .get_image_memory_requirements(texture_image)
        };

        let memory_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(image_memory_requirement.size)
            .memory_type_index(render_context.find_memory_type(
                image_memory_requirement.memory_type_bits,
                required_memory_properties,
            ))
            .build();

        let texture_image_memory = unsafe {
            render_context
                .device()
                .allocate_memory(&memory_allocate_info, None)
                .expect("Failed to allocate Texture Image memory!")
        };

        unsafe {
            render_context
                .device()
                .bind_image_memory(texture_image, texture_image_memory, 0)
                .expect("Failed to bind Image Memmory!");
        }

        (texture_image, texture_image_memory)
    }

    /// Transitions image layout.
    ///
    /// # Arguments
    /// - `old_layout`: The current layout of the image.
    /// - `new_layout`: The desired layout of the image.
    /// - `command_pool`: Command pool that is used to allocate a commandbuffer, to perform the transition, from.
    /// - `submit_queue`: The queue on which the commandbuffer will be queued.
    pub fn transition_image_layout(
        render_context: &RenderContext,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let command_buffer = begin_single_time_command(render_context);

        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask =
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        }];

        unsafe {
            render_context.device().cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers,
            );
        }

        end_single_time_command(render_context, &command_buffer);
    }

    fn copy_buffer_to_image(
        render_context: &RenderContext,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) {
        let command_buffer = begin_single_time_command(render_context);

        let buffer_image_regions = [vk::BufferImageCopy {
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            buffer_offset: 0,
            buffer_image_height: 0,
            buffer_row_length: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        }];

        unsafe {
            render_context.device().cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &buffer_image_regions,
            );
        }

        end_single_time_command(render_context, &command_buffer);
    }

    /// Creates a texture sampler that can be used to sample texel data from.
    pub fn create_texture_sampler(device: &VkLogicalDevice) -> vk::Sampler {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: device.device_properties().limits.max_sampler_anisotropy,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };

        unsafe {
            device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
    }

    pub unsafe fn destroy(&self, device: &VkLogicalDevice) {
        self.image_view.destroy(device);
        device.destroy_image(self.image, None);
        device.free_memory(self.device_memory, None);
    }
}
