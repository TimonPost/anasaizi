#![feature(array_map)]

#[macro_use]
pub mod debug;

pub mod vulkan;
pub mod utils;



use std::{
    ffi::{CStr, CString, IntoStringError},
    os::raw::c_char,
};

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;


