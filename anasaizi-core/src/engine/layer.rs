use crate::{
    engine::{Event, RenderContext, VulkanApplication},
    vulkan::{CommandPool, Queue, Window},
};

pub trait Layer {
    fn initialize(&mut self, window: &Window, render_context: &RenderContext);
    fn on_event(&mut self, event: &Event);

    fn start_frame(&mut self);
    fn on_update(
        &mut self,
        delta_time: u128,
        render_context: &RenderContext,
        application: &VulkanApplication,
    );
    fn end_frame(&mut self);
}
