use ash::{vk, Device};
use crate::surface::SurfaceData;
use crate::{LogicalDevice, Instance, WINDOW_WIDTH, WINDOW_HEIGHT};
use crate::QueueFamilyIndices;
use std::ptr;
use std::ops::Deref;
use ash::version::DeviceV1_0;

/// The swap chain is essentially a queue of images that are waiting to be presented to the screen.
/// Our application will acquire such an image to draw to it, and then return it to the queue.
/// How exactly the queue works and the conditions for presenting an image from the queue depend on how the swap chain is set up,
/// but the general purpose of the swap chain is to synchronize the presentation of images with the refresh rate of the screen.
pub struct SwapChain {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>
}

impl SwapChain {
    pub fn new(instance: &Instance, device: &LogicalDevice, surface_data: &SurfaceData) -> SwapChain{
        let swap_chain_support = Self::query_swapchain_support(device, surface_data);

        if swap_chain_support.formats.is_empty() && swap_chain_support.present_modes.is_empty() {
            panic!("Swapchain can not be configured with present modes!")
        }

        return Self::create_swapchain(instance, device, surface_data);
    }

    fn query_swapchain_support(
        device: &LogicalDevice,
        surface_data: &SurfaceData,
    ) -> SwapChainSupportDetails {
        unsafe {
            let capabilities = surface_data
                .surface_loader
                .get_physical_device_surface_capabilities(*device.physical_device(), surface_data.surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface_data
                .surface_loader
                .get_physical_device_surface_formats(*device.physical_device(), surface_data.surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface_data
                .surface_loader
                .get_physical_device_surface_present_modes(*device.physical_device(), surface_data.surface)
                .expect("Failed to query for surface present mode.");

            SwapChainSupportDetails {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    fn create_swapchain(
        instance: &Instance,
        device: &LogicalDevice,
        surface_stuff: &SurfaceData,
    ) -> SwapChain {
        let swapchain_support = Self::query_swapchain_support(device, surface_stuff);

        let surface_format = Self::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = Self::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Self::choose_swapchain_extent(&swapchain_support.capabilities);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let queue_family = device.queue_family_indices();

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
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

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface_stuff.surface,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
        };

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance.deref(), device.logical_device());
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

        let image_views = Self::create_image_views(&device, &swapchain_images, &surface_format.format);

        SwapChain {
            loader: swapchain_loader,
            swapchain,
            image_format: surface_format.format,
            extent: extent,
            image_views,
            images: swapchain_images,
        }
    }

    fn create_image_views(device: &LogicalDevice, images: &Vec<vk::Image>, format: &vk::Format) -> Vec<vk::ImageView> {
        let mut swapchain_imageviews = vec![];

        for &image in images.iter() {
            let imageview_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                view_type: vk::ImageViewType::TYPE_2D,
                format: *format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image,
            };

            let imageview = unsafe {
                device.logical_device()
                    .create_image_view(&imageview_create_info, None)
                    .expect("Failed to create Image View!")
            };
            swapchain_imageviews.push(imageview);
        }

        swapchain_imageviews
    }

    /// Pick a format to use for the swapchain.
    fn choose_swapchain_format(available_formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
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
    present_modes: Vec<vk::PresentModeKHR>
}