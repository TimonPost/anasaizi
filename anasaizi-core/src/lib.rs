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

pub mod libs {
    pub mod nalgebra {
        pub use nalgebra::*;
    }

    pub mod imgui {
        pub use imgui::*;
    }

    pub mod imgui_winit_support {
        pub use imgui_winit_support::*;
    }

    pub mod image {
        pub use image::*;
    }

    pub mod tokio {
        pub use tokio::*;
    }

    pub mod ash {
        pub use ash::*;
    }

    pub mod futures {
        pub use futures::*;
    }

    pub mod hecs {
        pub use hecs::*;
    }
}
