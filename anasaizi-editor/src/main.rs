use crate::sandbox::VulkanApp;

use anasaizi_profile::profile;

use anasaizi_core::libs::tokio;
use std::io;
use winit::event_loop::EventLoop;

#[macro_use]
mod sandbox;
mod game_layer;
mod imgui_layer;

#[tokio::main]
#[profile]
async fn main() -> io::Result<()> {
    let event_loop = EventLoop::new();

    let app = VulkanApp::new(&event_loop).await;
    app.main_loop(event_loop);

    Ok(())
}
