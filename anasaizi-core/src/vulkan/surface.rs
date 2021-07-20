use ash::extensions::khr::Win32Surface;

use crate::vulkan::Instance;
use ash::vk;
use std::ops::Deref;

/// In Vulkan, the windowing system particulars are exposed via the WSI (Window System Integration) extensions.
///
/// Vulkan uses the VkSurfaceKHR object to abstract the native platform surface or window.
/// This symbol is defined as part of the VK_KHR_surface extension.
/// The various functions in the WSI extensions are used to create, manipulate, and destroy these surface objects.
pub struct SurfaceData {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
}

impl SurfaceData {
    pub fn new(instance: &Instance, window: &winit::window::Window) -> SurfaceData {
        let surface = unsafe {
            Self::create_surface_windows(instance, window).expect("Failed to create surface.")
        };

        let surface_loader = ash::extensions::khr::Surface::new(instance.entry(), instance.deref());

        SurfaceData {
            surface_loader,
            surface,
        }
    }

    #[cfg(target_os = "windows")]
    pub unsafe fn create_surface_windows(
        instance: &Instance,
        window: &winit::window::Window,
    ) -> Result<vk::SurfaceKHR, vk::Result> {
        use std::{os::raw::c_void, ptr};
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winit::platform::windows::WindowExtWindows;

        let hwnd = window.hwnd();
        let hinstance = GetModuleHandleW(ptr::null()) as *const c_void;
        let win32_create_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(hinstance)
            .hwnd(hwnd);

        let win32_surface_loader = Win32Surface::new(instance.entry(), instance.deref());
        win32_surface_loader.create_win32_surface(&win32_create_info, None)
    }
}
