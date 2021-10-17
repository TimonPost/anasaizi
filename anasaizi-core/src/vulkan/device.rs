use ash::{version::InstanceV1_0, vk};

use std::fmt::Formatter;

use crate::{
    utils::vk_to_string,
    vulkan::{Version, VkInstance, VkQueueFamilyProperties, VkSurfaceData},
};

use crate::{engine::Extensions, vulkan::VkQueueFamilyIndices};
use std::{fmt, ops::Deref};

/// A Vulkan logical device.
///
/// A logical device acts upont a physical device by proving helper functions.
#[derive(Clone)]
pub struct VkLogicalDevice {
    physical_device: vk::PhysicalDevice,
    device_features: vk::PhysicalDeviceFeatures,
    device_properties: DeviceProperties,
    queue_family_indices: VkQueueFamilyIndices,
    logical_device: ash::Device,
    device_mem_properties: vk::PhysicalDeviceMemoryProperties,
}

impl VkLogicalDevice {
    pub fn new(
        instance: &VkInstance,
        required_extensions: Extensions,
        surface_data: &VkSurfaceData,
    ) -> VkLogicalDevice {
        let (queue_family_indices, physical_device) =
            Self::pick_physical_device(instance, surface_data);

        if !Self::check_device_extension_support(instance, physical_device, &required_extensions) {
            panic!(
                "Device does not support the extensions: {:?}",
                required_extensions
            );
        }

        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let device_features = unsafe { instance.get_physical_device_features(physical_device) };

        let device_properties = DeviceProperties {
            device_name: vk_to_string(&device_properties.device_name).unwrap(),
            device_type: DeviceType::from(device_properties.device_type),
            api_version: Version::decode(device_properties.api_version),
            driver_version: Version::decode(device_properties.driver_version),
            vendor_id: device_properties.vendor_id,
            device_id: device_properties.device_id,
            limits: device_properties.limits,
        };

        let logical_device = Self::create_logical_device(
            &instance,
            physical_device,
            &surface_data,
            &required_extensions,
            device_features,
        );

        let device_mem_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        VkLogicalDevice {
            physical_device,
            device_properties,
            device_features,
            queue_family_indices,
            logical_device,
            device_mem_properties,
        }
    }

    pub fn find_memory_type(
        &self,
        type_filter: u32,
        required_properties: vk::MemoryPropertyFlags,
    ) -> u32 {
        for (i, memory_type) in self.device_mem_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) > 0
                && memory_type.property_flags.contains(required_properties)
            {
                return i as u32;
            }
        }

        panic!("Failed to find suitable memory type!")
    }

    pub fn find_depth_format(&self, instance: &ash::Instance) -> vk::Format {
        self.find_supported_format(
            instance,
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn find_supported_format(
        &self,
        instance: &ash::Instance,
        candidate_formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for &format in candidate_formats.iter() {
            let format_properties = unsafe {
                instance.get_physical_device_format_properties(self.physical_device, format)
            };
            if tiling == vk::ImageTiling::LINEAR
                && format_properties.linear_tiling_features.contains(features)
            {
                return format.clone();
            } else if tiling == vk::ImageTiling::OPTIMAL
                && format_properties.optimal_tiling_features.contains(features)
            {
                return format.clone();
            }
        }

        panic!("Failed to find supported format!")
    }

    pub fn device_properties(&self) -> &DeviceProperties {
        &self.device_properties
    }

    pub fn device_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        self.device_mem_properties.clone()
    }

    pub fn physical_device(&self) -> &vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn logical_device(&self) -> &ash::Device {
        &self.logical_device
    }

    pub fn queue_family_indices(&self) -> &VkQueueFamilyIndices {
        &self.queue_family_indices
    }

    fn create_logical_device(
        instance: &VkInstance,
        physical_device: vk::PhysicalDevice,
        surface_data: &VkSurfaceData,
        extensions: &Extensions,
        features: vk::PhysicalDeviceFeatures,
    ) -> ash::Device {
        // Setup the queues to use
        let indices = Self::find_queue_family(instance, physical_device, surface_data);

        let queue_priorities = [1.0_f32];

        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.graphics_family.unwrap())
            .queue_priorities(&queue_priorities)
            .build();

        // Get extensions
        let extensions_raw = extensions.as_cstrings();
        let extensions_ptr = extensions_raw
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<*const i8>>();

        // Create device
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&[queue_create_info])
            .enabled_extension_names(&extensions_ptr)
            .enabled_features(&features)
            .build();

        let device: ash::Device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };

        device
    }

    /// Validates if the physical device supports the required extensions.
    fn check_device_extension_support(
        instance: &VkInstance,
        physical_device: vk::PhysicalDevice,
        required_extensions: &Extensions,
    ) -> bool {
        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to get device extension properties.")
        }
        .iter()
        .map(|e| vk_to_string(&e.extension_name))
        .filter(|result| result.is_ok())
        .map(|string| string.unwrap())
        .collect::<Vec<String>>();

        let available_extensions = Extensions::new(available_extensions);

        let result = available_extensions.has(&required_extensions);

        result
    }

    /// Pick a physical device that is capable of using the graphics queue.
    fn pick_physical_device(
        instance: &VkInstance,
        surface_data: &VkSurfaceData,
    ) -> (VkQueueFamilyIndices, vk::PhysicalDevice) {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices!")
        };

        let mut result = None;
        for &physical_device in physical_devices.iter() {
            let (queue_family, suitable) =
                Self::is_physical_device_suitable(instance, physical_device, surface_data);

            if suitable {
                if result.is_none() {
                    result = Some((queue_family, physical_device))
                }
            }
        }

        match result {
            None => panic!("Failed to find a suitable GPU!"),
            Some(physical_device) => physical_device,
        }
    }

    /// Validate if the physical device is able to use the graphics queue.
    fn is_physical_device_suitable(
        instance: &VkInstance,
        physical_device: vk::PhysicalDevice,
        surface_data: &VkSurfaceData,
    ) -> (VkQueueFamilyIndices, bool) {
        let indices = Self::find_queue_family(instance, physical_device, surface_data);
        let is_complete = indices.is_complete();

        return (indices, is_complete);
    }

    /// Find supported queues for the given device.
    fn find_queue_family(
        instance: &VkInstance,
        physical_device: vk::PhysicalDevice,
        surface_data: &VkSurfaceData,
    ) -> VkQueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = VkQueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };

        let mut index = 0;

        for queue_family in queue_families.iter() {
            let family = VkQueueFamilyProperties::from(*queue_family);

            if family.is_graphics() {
                queue_family_indices.graphics_family = Some(index);
            }

            let is_present_support = unsafe {
                surface_data
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface_data.surface,
                    )
            }
            .expect("Error when trying to check present support");

            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }
}

impl fmt::Debug for VkLogicalDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\nDevice Features: {:?}\n", self.device_features)?;
        write!(f, "\n{:?}\n", self.device_properties)
    }
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    Other,
    IntegratedGPU,
    DiscreteGPU,
    VirtualGPU,
    CPU,
}

impl From<vk::PhysicalDeviceType> for DeviceType {
    fn from(device_type: vk::PhysicalDeviceType) -> Self {
        match device_type {
            vk::PhysicalDeviceType::CPU => DeviceType::CPU,
            vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceType::IntegratedGPU,
            vk::PhysicalDeviceType::DISCRETE_GPU => DeviceType::DiscreteGPU,
            vk::PhysicalDeviceType::VIRTUAL_GPU => DeviceType::VirtualGPU,
            vk::PhysicalDeviceType::OTHER => DeviceType::DiscreteGPU,
            _ => panic!(),
        }
    }
}

#[derive(Clone)]
pub struct DeviceProperties {
    pub api_version: Version,
    pub driver_version: Version,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: DeviceType,
    pub device_name: String,
    pub limits: vk::PhysicalDeviceLimits,
}

impl fmt::Debug for DeviceProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\nDevice Properties:\n")?;
        write!(
            f,
            "\t- Device Name: {}, id: {}, type: {:?}\n",
            self.device_name, self.device_id, self.device_type
        )?;
        write!(f, "\t- API Version: {:?}\n", self.api_version)?;
        write!(f, "\t- Driver Version: {:?}\n", self.driver_version)?;

        Ok(())
    }
}

impl Deref for VkLogicalDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.logical_device
    }
}
