use crate::{
    engine::Extensions,
    libs::imgui::__core::fmt::Formatter,
    vulkan::{
        structures::VkValidationInfo, Version, VkApplication, VkInstance, VkLogicalDevice, Window,
    },
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use ash::extensions::{ext::DebugUtils, khr};
use std::{fmt, fmt::Debug};
use winit::event_loop::EventLoop;

pub const VALIDATION: VkValidationInfo = VkValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

/// Vulkan application with winit window, vulkan data such as instance, device and application.
pub struct VulkanApplication {
    pub application: VkApplication,
    pub instance: VkInstance,
    pub window: Window,
    pub device: VkLogicalDevice,
}

impl VulkanApplication {
    /// Creates a new vulkan application.
    ///
    /// This function configures:
    /// - Application
    /// - Instance
    /// - LogicalDevice
    /// - Device/Instance Extensions
    pub fn new(name: &'static str, event_loop: &EventLoop<()>) -> Self {
        let instance_extensions = Extensions::new(vec![
            khr::Surface::name().to_str().unwrap().to_string(),
            khr::Win32Surface::name().to_str().unwrap().to_string(),
            DebugUtils::name().to_str().unwrap().to_string(),
        ]);

        let device_extensions =
            Extensions::new(vec![khr::Swapchain::name().to_str().unwrap().to_string()]);

        let application = VkApplication::new(
            name,
            name,
            Version::new(0, 0, 1),
            Version::new(1, 2, 0),
            Version::new(1, 2, 0),
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );

        let instance = VkInstance::new(VALIDATION, instance_extensions, &application);

        let window = Window::new("Engine", WINDOW_WIDTH, WINDOW_HEIGHT, &instance, event_loop);

        let device = VkLogicalDevice::new(&instance, device_extensions, window.surface_data());

        VulkanApplication {
            application,
            instance,
            window,
            device,
        }
    }
}

impl Debug for VulkanApplication {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.application)?;
        write!(f, "{:?}", self.device)
    }
}
