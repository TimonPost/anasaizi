use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, ModifiersState, MouseButton},
};

pub enum Event {
    Mouse(ModifiersState, ElementState, MouseButton),
    MouseMove(PhysicalPosition<f64>),
    MouseScroll(f32, f32),
    Keyboard(KeyboardInput),
}
