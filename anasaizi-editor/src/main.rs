use crate::sandbox::VulkanApp;
use winit::event_loop::EventLoop;

mod sandbox;

fn main() {
    let event_loop = EventLoop::new();

    let app = VulkanApp::new(&event_loop);
    app.main_loop(event_loop);
}
