use std::sync::Arc;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, ModifiersState, MouseButton},
};

#[derive(Clone)]
pub enum Event {
    Mouse(ModifiersState, ElementState, MouseButton),
    MouseMove(PhysicalPosition<f64>, ModifiersState),
    MouseScroll(f32, f32),
    Keyboard(KeyboardInput),
    MouseInput(ElementState, MouseButton),
    Raw(Arc<winit::event::Event<'static, ()>>),
    Shutdown,
}
