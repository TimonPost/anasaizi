use crate::{
    vulkan::{
        structures::ValidationInfo, Application, Extensions, Instance, LogicalDevice, Version,
        Window,
    },
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use ash::extensions::{ext::DebugUtils, khr};
use winit::event_loop::EventLoop;

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub struct VulkanApplication {
    pub device: LogicalDevice,
    pub application: Application,
    pub instance: Instance,
    pub window: Window,
}

impl VulkanApplication {
    pub fn new(name: &'static str, event_loop: &EventLoop<()>) -> Self {
        let instance_extensions = Extensions::new(vec![
            khr::Surface::name().to_str().unwrap().to_string(),
            khr::Win32Surface::name().to_str().unwrap().to_string(),
            DebugUtils::name().to_str().unwrap().to_string(),
        ]);

        let device_extensions =
            Extensions::new(vec![khr::Swapchain::name().to_str().unwrap().to_string()]);

        let application = Application::new(
            name,
            name,
            Version::new(0, 0, 1),
            Version::new(1, 2, 0),
            Version::new(1, 2, 0),
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );

        let instance = Instance::new(VALIDATION, instance_extensions, &application);

        let window = Window::new("Engine", WINDOW_WIDTH, WINDOW_HEIGHT, &instance, event_loop);

        let device = LogicalDevice::new(&instance, device_extensions, window.surface_data());

        VulkanApplication {
            application,
            instance,
            window,
            device,
        }
    }
}