#![feature(array_map)]
#![feature(drain_filter)]

#[macro_use]
pub mod debug;

mod application;
mod device;
mod extensions;
mod instance;
mod layer;
mod queue;
pub mod structures;
mod surface;
mod version;
mod window;
mod swapchain;
mod render_pass;
mod shader;
mod pipeline;
mod framebuffer;
mod command_pool;
mod command_buffer;


pub use application::Application;
pub use command_pool::{CommandPool};
pub use command_buffer::CommandBuffers;
pub use device::LogicalDevice;
pub use extensions::Extensions;
pub use framebuffer::{FrameBuffer, FrameBuffers};
pub use layer::{ValidationLayers, ValidationLayerProperties};
pub use pipeline::{Pipeline};
pub use queue::{Queue, QueueFamilyProperties};
pub use render_pass::RenderPass;
pub use shader::{Shader, Shaders};
pub use structures::QueueFamilyIndices;
pub use surface::SurfaceData;
pub use swapchain::{SwapChain, SwapChainSupportDetails};
pub use version::Version;
pub use instance::Instance;
pub use window::Window;

use std::ffi::{CString, IntoStringError, CStr};
use std::os::raw::c_char;
use std::time::{Instant, SystemTime};
use lazy_static::lazy_static;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub fn vk_to_cstr(raw_string_array: &[c_char]) -> CString {
    // Implementation 2
    unsafe {
        let mut pointer = raw_string_array.as_ptr() as *mut c_char;
        CString::from_raw(pointer)
    }
}

pub fn vk_to_string(raw_string_array: &[c_char]) -> Result<String, IntoStringError> {
    let c_str = unsafe { CStr::from_ptr(raw_string_array.as_ptr()) };
    let c_string = c_str.to_owned();
    let string = c_string.into_string();

    string
}



