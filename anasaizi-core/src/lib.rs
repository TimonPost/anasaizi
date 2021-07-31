#![feature(array_map)]

#[macro_use]
pub mod debug;

pub mod engine;
pub mod math;
pub mod model;
pub mod utils;
pub mod vulkan;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub mod reexports {
    pub mod nalgebra {
        pub use nalgebra::*;
    }

    pub mod imgui {
        pub use imgui::*;
    }

    pub mod imgui_winit_support {
        pub use imgui_winit_support::*;
    }
}
