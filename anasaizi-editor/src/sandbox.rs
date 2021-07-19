use anasaizi_core::vulkan::{
    IndexBuffer, LogicalDevice, ShaderSet, UniformBufferObject, VertexBuffer,
};
use ash::vk;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use anasaizi_core::{
    engine,
};
use anasaizi_profile::profile;

use anasaizi_core::{
    debug::{start_profiler, stop_profiler},
    engine::{image::Texture, VulkanApplication, VulkanRenderer, FRAGMENT_SHADER, VERTEX_SHADER},
    math::Vertex,
    model::{Mesh, Object},
};
use ash::{
    version::DeviceV1_0,
};
use std::{path::Path};
use winit::event::MouseScrollDelta;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct VulkanApp {
    vulkan_renderer: VulkanRenderer,
    application: VulkanApplication,

    pub viking_indices: Vec<u32>,
    pub viking_vertices: Vec<Vertex>,
    pub viking_room_texture: Texture,
}

impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let application = VulkanApplication::new("Vulkan Engine", event_loop);

        let mut vulkan_renderer = VulkanRenderer::new(&application);

        let viking_room_texture = Texture::create(
            &application.instance,
            &application.device,
            &vulkan_renderer.command_pool,
            &vulkan_renderer.graphics_queue,
            &Path::new("viking_room.png"),
        );
        let (viking_vertices, viking_indices) = Object::load_model(Path::new("viking_room.obj"));

        let descriptor_layout = Self::descriptor_set_layout(&application.device);

        let shaders = ShaderSet::<UniformBufferObject>::new(
            &application.instance,
            &application.device,
            VERTEX_SHADER,
            FRAGMENT_SHADER,
            descriptor_layout,
            3,
            vulkan_renderer.texture_sampler.unwrap(),
            &viking_room_texture,
        );

        let vertex_buffer = VertexBuffer::create(
            &application.instance,
            &application.device,
            &viking_vertices,
            &vulkan_renderer.graphics_queue,
            &vulkan_renderer.command_pool,
        );
        let index_buffer = IndexBuffer::create(
            &application.instance,
            &application.device,
            &viking_indices,
            &vulkan_renderer.graphics_queue,
            &vulkan_renderer.command_pool,
        );
        let mesh = Mesh::new(vertex_buffer, index_buffer);

        vulkan_renderer.setup_test(&application.device, &shaders, &mesh);

        start_profiler();

        VulkanApp {
            vulkan_renderer,
            application,

            viking_room_texture,
            viking_vertices,
            viking_indices,
        }
    }

    pub fn descriptor_set_layout(device: &LogicalDevice) -> vk::DescriptorSetLayout {
        let layout_binding = [
            vk::DescriptorSetLayoutBinding::builder()
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .binding(0)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .binding(1)
                .build(),
        ];

        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layout_binding)
            .build();

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_create_info, None)
                .expect("failed to create descriptor set layout!")
        };

        descriptor_set_layout
    }

    fn update_uniform(&mut self, _current_image: usize) {
        // let rotation = cgmath::Matrix4::<f32>::from_axis_angle(cgmath::Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * 0.0001);
        //
        // let ubos = [self.uniform_buffer_object.clone()];
        //
        // let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;
        //
        // unsafe {
        //     let data_ptr =
        //         self.device
        //             .map_memory(
        //                 self.uniform_buffer.buffers_memory(current_image),
        //                 0,
        //                 buffer_size,
        //                 vk::MemoryMapFlags::empty(),
        //             )
        //             .expect("Failed to Map Memory") as *mut UniformBufferObject;
        //
        //     data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());
        //
        //     self.device
        //         .unmap_memory(self.uniform_buffer.buffers_memory(current_image));
        // }
    }

    #[profile(Sandbox)]
    fn draw_frame(&mut self) {
        self.vulkan_renderer.draw(&self.application);
    }

    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;

                    stop_profiler();
                }
                WindowEvent::CursorMoved { position, .. } => self
                    .vulkan_renderer
                    .handle_event(engine::Event::MouseMove(position)),
                WindowEvent::MouseWheel { delta, .. } => {
                    if let MouseScrollDelta::LineDelta(x, y) = delta {
                        self.vulkan_renderer
                            .handle_event(engine::Event::MouseScroll(x, y));
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    self.vulkan_renderer
                        .handle_event(engine::Event::Keyboard(input));
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                self.application.window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                self.draw_frame();
            }
            Event::LoopDestroyed => {
                unsafe {
                    self.application
                        .device
                        .device_wait_idle()
                        .expect("Failed to wait device idle!")
                };
            }
            _ => (),
        })
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        // unsafe {
        //     // self.device.destroy_shader_module(vert_shader_module, None);
        //     // self.device.destroy_shader_module(frag_shader_module, None);
        //
        //     for &imageview in self.image_views.iter() {
        //         self.device.destroy_image_view(imageview, None);
        //     }
        //
        //     self.swapchain_data
        //         .swapchain_loader
        //         .destroy_swapchain(self.swapchain_data.swapchain, None);
        //     self.device.destroy_device(None);
        //     self.surface_data
        //         .surface_loader
        //         .destroy_surface(self.surface_data.surface, None);
        //
        //     if VALIDATION.is_enable {
        //         self.debug_utils_loader
        //             .destroy_debug_utils_messenger(self.debug_merssager, None);
        //     }
        //     self.instance.destroy_instance(None);
        // }
    }
}
