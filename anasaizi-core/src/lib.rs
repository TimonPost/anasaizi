#![feature(array_map)]

#[macro_use]
pub mod debug;

pub mod utils;
pub mod vulkan;
pub mod math;
pub mod model;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
