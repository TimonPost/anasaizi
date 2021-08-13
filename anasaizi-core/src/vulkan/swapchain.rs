use ash::vk;

use crate::{
    engine::{image::Texture, RenderContext},
    vulkan::{ImageView, Instance, LogicalDevice, SurfaceData},
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use ash::vk::Image;
use std::ops::Deref;

/// A Vulkan Swapchain.
///
/// The swap chain is essentially a queue of images that are waiting to be presented to the screen.
/// The general purpose of the swap chain is to synchronize the presentation of images with the refresh rate of the screen.
pub struct SwapChain {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<ImageView>,
    pub depth_image: vk::Image,
    pub depth_image_view: ImageView,
}

impl SwapChain {
    pub(crate) unsafe fn destroy(&self, device: &LogicalDevice) {
        self.depth_image_view.destroy(device);
        for image_view in self.image_views.iter() {
            image_view.destroy(device);
        }

        self.loader.destroy_swapchain(self.swapchain, None);
    }
}

impl SwapChain {
    pub fn new(render_context: &RenderContext, surface_data: &SurfaceData) -> SwapChain {
        let swap_chain_support =
            Self::query_swapchain_support(render_context.physical_device(), surface_data);

        if swap_chain_support.formats.is_empty() && swap_chain_support.present_modes.is_empty() {
            panic!("Swapchain can not be configured with present modes!")
        }

        return Self::create_swapchain(render_context, surface_data);
    }

    fn query_swapchain_support(
        physical_device: &vk::PhysicalDevice,
        surface_data: &SurfaceData,
    ) -> SwapChainSupportDetails {
        unsafe {
            let capabilities = surface_data
                .surface_loader
                .get_physical_device_surface_capabilities(*physical_device, surface_data.surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface_data
                .surface_loader
                .get_physical_device_surface_formats(*physical_device, surface_data.surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface_data
                .surface_loader
                .get_physical_device_surface_present_modes(*physical_device, surface_data.surface)
                .expect("Failed to query for surface present mode.");

            SwapChainSupportDetails {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    fn create_swapchain(render_context: &RenderContext, surface_stuff: &SurfaceData) -> SwapChain {
        let swapchain_support =
            Self::query_swapchain_support(render_context.physical_device(), surface_stuff);

        let surface_format = Self::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = Self::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Self::choose_swapchain_extent(&swapchain_support.capabilities);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let queue_family = render_context.logical_device().queue_family_indices();

        let (image_sharing_mode, _queue_family_index_count, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (
                    vk::SharingMode::EXCLUSIVE,
                    2,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface_stuff.surface)
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null())
            .image_array_layers(1)
            .build();

        let swapchain_loader = ash::extensions::khr::Swapchain::new(
            render_context.raw_instance(),
            render_context.device(),
        );
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.")
        };

        let depth_image = Self::create_depth_resources(render_context, extent);

        let image_views = Self::create_image_views(
            render_context.device(),
            &swapchain_images,
            &surface_format.format,
        );

        SwapChain {
            loader: swapchain_loader,
            swapchain,
            image_format: surface_format.format,
            extent,
            image_views,
            images: swapchain_images,
            depth_image: depth_image.0,
            depth_image_view: depth_image.1,
        }
    }

    /// An image view defines how the swapchain is going to use an image.
    fn create_image_views(
        device: &ash::Device,
        images: &Vec<vk::Image>,
        format: &vk::Format,
    ) -> Vec<ImageView> {
        let mut swapchain_imageviews = vec![];

        for &image in images.iter() {
            let image_view =
                ImageView::create(&device, image, *format, vk::ImageAspectFlags::COLOR);

            swapchain_imageviews.push(image_view);
        }

        swapchain_imageviews
    }

    fn create_depth_resources(
        render_context: &RenderContext,
        swapchain_extent: vk::Extent2D,
    ) -> (Image, ImageView, vk::DeviceMemory) {
        let depth_format = render_context
            .logical_device()
            .find_depth_format(render_context.raw_instance());

        let (depth_image, depth_image_memory) = Texture::create_image(
            render_context,
            swapchain_extent.width,
            swapchain_extent.height,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let depth_image_view = ImageView::create(
            render_context.device(),
            depth_image,
            depth_format,
            vk::ImageAspectFlags::DEPTH,
        );

        (depth_image, depth_image_view, depth_image_memory)
    }

    /// Pick a format to use for the swapchain.
    fn choose_swapchain_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        // check if list contains most widely used R8G8B8A8 format with nonlinear color space
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        // return the first format from the list
        return available_formats.first().unwrap().clone();
    }

    /// Pick mailbox present mode if possible, otherwise pick FIFO.
    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    /// Create extend 2d
    fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            use num::clamp;

            vk::Extent2D {
                width: clamp(
                    WINDOW_WIDTH,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    WINDOW_HEIGHT,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}

impl Deref for SwapChain {
    type Target = vk::SwapchainKHR;

    fn deref(&self) -> &Self::Target {
        &self.swapchain
    }
}

pub struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}
