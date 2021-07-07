use ash::extensions::khr::Win32Surface;

use ash::vk;
use std::ops::Deref;
use crate::vulkan::Instance;

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
        use winapi::{shared::windef::HWND, um::libloaderapi::GetModuleHandleW};
        use winit::platform::windows::WindowExtWindows;

        let hwnd = window.hwnd() as HWND;
        let hinstance = GetModuleHandleW(ptr::null()) as *const c_void;
        let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: Default::default(),
            hinstance,
            hwnd: hwnd as *const c_void,
        };
        let win32_surface_loader = Win32Surface::new(instance.entry(), instance.deref());
        win32_surface_loader.create_win32_surface(&win32_create_info, None)
    }
}
