use anasaizi_core::vulkan::{
    BufferLayout, IndexBuffer, LogicalDevice, ShaderBuilder, ShaderSet, UniformBufferObject,
    VertexBuffer,
};
use ash::vk;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use anasaizi_core::{engine, reexports::nalgebra as math, WINDOW_HEIGHT, WINDOW_WIDTH};

use anasaizi_core::debug::*;
use anasaizi_profile::profile;

use anasaizi_core::{
    debug::{start_profiler, stop_profiler},
    engine::{image::Texture, VulkanApplication, VulkanRenderer, FRAGMENT_SHADER, VERTEX_SHADER},
    math::Vertex,
    model::{square_indices, square_vertices, Mesh, Object},
};
use ash::version::DeviceV1_0;
use std::path::Path;
use winit::event::MouseScrollDelta;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct VulkanApp {
    vulkan_renderer: VulkanRenderer<UniformBufferObject>,
    application: VulkanApplication,

    pub viking_indices: Vec<u32>,
    pub viking_vertices: Vec<Vertex>,
    pub viking_room_texture: [Texture; 1],

    count: f32,
}

impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let application = VulkanApplication::new("Vulkan Engine", event_loop);

        let mut vulkan_renderer = VulkanRenderer::new(&application);

        let (viking_vertices, viking_indices) = Object::load_model(Path::new("viking_room.obj"));

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

        let viking_room_texture = [Texture::create(
            &application.instance,
            &application.device,
            &vulkan_renderer.command_pool,
            &vulkan_renderer.graphics_queue,
            &Path::new("viking_room.png"),
        )];

        let shader_set = Self::setup_shader(&application, &vulkan_renderer, &viking_room_texture);

        vulkan_renderer.push_render_object(&application, shader_set, mesh);

        start_profiler();

        VulkanApp {
            vulkan_renderer,
            application,

            viking_room_texture,
            viking_vertices,
            viking_indices,
            count: 0.0,
        }
    }

    pub fn setup_shader(
        application: &VulkanApplication,
        vulkan_renderer: &VulkanRenderer<UniformBufferObject>,
        texture: &[Texture],
    ) -> ShaderSet<UniformBufferObject> {
        let input_buffer_layout = BufferLayout::new()
            .add_float_vec3(0)
            .add_float_vec3(1)
            .add_float_vec2(2);

        let descriptor_image_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(*texture[0].image_view)
            .sampler(vulkan_renderer.texture_sampler.unwrap())
            .build()];

        let mut descriptor_write_sets = vec![
            vk::WriteDescriptorSet::builder()
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_array_element(0)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_array_element(0)
                .image_info(&descriptor_image_info)
                .build(),
        ];

        let mut builder = ShaderBuilder::builder(application, VERTEX_SHADER, FRAGMENT_SHADER, 3);
        builder
            .with_textures(&texture, vulkan_renderer.texture_sampler.unwrap())
            .with_input_buffer_layout(input_buffer_layout)
            .with_write_descriptor_layout(&Self::descriptor_set_layout(&application.device))
            .with_descriptor_pool(&[
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            ])
            .with_write_descriptor_sets(descriptor_write_sets);

        builder.build()
    }

    pub fn descriptor_set_layout(device: &LogicalDevice) -> [vk::DescriptorSetLayoutBinding; 2] {
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

        layout_binding
    }

    #[profile(Sandbox)]
    fn update_uniform(&mut self, _current_image: usize) {
        self.count += 1.0 / 1000.0;

        let (view, perspective) = {
            let camera = self.vulkan_renderer.camera();
            camera.reload();
            (camera.view(), camera.projection())
        };

        for render_object in self.vulkan_renderer.render_objects.iter_mut() {
            let uniform_mut = render_object.shader.uniform_mut();

            //if camera.is_dirty() {

            let rotation = math::Matrix4::new_rotation(math::Vector3::new(0.0, self.count, 0.0));

            uniform_mut.view = view;
            uniform_mut.proj = perspective;
            // }
            uniform_mut.model = rotation;

            render_object
                .shader
                .update_uniform(&self.application.device, _current_image);
        }
    }

    #[profile(Sandbox)]
    fn draw_frame(&mut self) {
        self.update_uniform(self.vulkan_renderer.current_frame());
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
