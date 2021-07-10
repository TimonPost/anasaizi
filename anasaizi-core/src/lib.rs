#![feature(array_map)]

#[macro_use]
pub mod debug;

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
}
