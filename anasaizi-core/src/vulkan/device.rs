use ash::{version::InstanceV1_0, vk, vk::PhysicalDeviceFeatures};

use std::fmt::Formatter;

use std::{fmt, ops::Deref, ptr};
use crate::vulkan::{Version, QueueFamilyIndices, SurfaceData, Extensions, Instance, QueueFamilyProperties};
use crate::utils::vk_to_string;

pub struct LogicalDevice {
    physical_device: vk::PhysicalDevice,
    device_features: vk::PhysicalDeviceFeatures,
    device_properties: DeviceProperties,
    queue_family_indices: QueueFamilyIndices,
    logical_device: ash::Device,
}

impl LogicalDevice {
    pub fn new(
        instance: &Instance,
        required_extensions: Extensions,
        surface_data: &SurfaceData,
    ) -> LogicalDevice {
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
        };

        let logical_device = Self::create_logical_device(
            &instance,
            physical_device,
            &surface_data,
            &required_extensions,
            device_features,
        );

        LogicalDevice {
            physical_device,
            device_properties,
            device_features,
            queue_family_indices,
            logical_device,
        }
    }

    pub fn physical_device(&self) -> &vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn logical_device(&self) -> &ash::Device {
        &self.logical_device
    }

    pub fn queue_family_indices(&self) -> &QueueFamilyIndices {
        &self.queue_family_indices
    }

    pub fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_data: &SurfaceData,
        extensions: &Extensions,
        features: PhysicalDeviceFeatures,
    ) -> ash::Device {
        // Setup the queues to use
        let indices = Self::find_queue_family(instance, physical_device, surface_data);

        let queue_priorities = [1.0_f32];

        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: indices.graphics_family.unwrap(),
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };

        // Get extensions
        let extensions_raw = extensions.extensions_ptr();
        let extensions_ptr = extensions_raw
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<*const i8>>();

        // Create device
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: extensions.extensions_count(),
            pp_enabled_extension_names: extensions_ptr.as_ptr(),
            p_enabled_features: &features,
        };

        let device: ash::Device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };

        device
    }

    /// Validates if the physical device supports the required extensions.
    fn check_device_extension_support(
        instance: &Instance,
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
        instance: &Instance,
        surface_data: &SurfaceData,
    ) -> (QueueFamilyIndices, vk::PhysicalDevice) {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices!")
        };

        println!(
            "{} devices (GPU) found with vulkan support.",
            physical_devices.len()
        );

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
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_data: &SurfaceData,
    ) -> (QueueFamilyIndices, bool) {
        let indices = Self::find_queue_family(instance, physical_device, surface_data);
        let is_complete = indices.is_complete();

        return (indices, is_complete);
    }

    /// Find supported queues for the given device.
    fn find_queue_family(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_data: &SurfaceData,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };

        let mut index = 0;

        println!("\n\tSupport Queue Family: {}", queue_families.len());

        for queue_family in queue_families.iter() {
            let family = QueueFamilyProperties::from(*queue_family);

            println!("{:?}", family);

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

impl fmt::Debug for LogicalDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\nDevice Features: {:?}\n", self.device_features)?;
        write!(f, "\n{:?}\n", self.device_properties)
    }
}

#[derive(Debug)]
enum DeviceType {
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

struct DeviceProperties {
    api_version: Version,
    driver_version: Version,
    vendor_id: u32,
    device_id: u32,
    device_type: DeviceType,
    device_name: String,
}

impl fmt::Debug for DeviceProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\nDevice Properties:\n");
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

impl Deref for LogicalDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.logical_device
    }
}
