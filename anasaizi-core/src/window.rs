use crate::{surface::SurfaceData, Instance};
use std::ops::Deref;
use winit::event_loop::EventLoop;

pub struct Window {
    surface: SurfaceData,
    window: winit::window::Window,
}

impl Window {
    pub fn new(
        window_title: &str,
        window_width: u32,
        window_height: u32,
        instance: &Instance,
        event_loop: &EventLoop<()>,
    ) -> Window {
        let window = winit::window::WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height))
            .build(event_loop)
            .expect("Failed to create window.");

        let surface = SurfaceData::new(instance, &window);

        Window { window, surface }
    }

    pub fn surface_data(&self) -> &SurfaceData {
        &self.surface
    }
}

impl Deref for Window {
    type Target = winit::window::Window;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
