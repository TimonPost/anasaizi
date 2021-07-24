use crate::sandbox::VulkanApp;

use anasaizi_core::debug::*;
use anasaizi_profile::profile;

use winit::event_loop::EventLoop;

#[macro_use]
mod sandbox;

#[profile]
fn main() {
    let event_loop = EventLoop::new();

    let app = VulkanApp::new(&event_loop);
    app.main_loop(event_loop);
}
