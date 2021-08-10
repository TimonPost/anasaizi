use anasaizi_core::vulkan::{
    CommandPool, IndexBuffer, Instance, LogicalDevice, MeshPushConstants, Pipeline, Queue,
    ShaderBuilder, ShaderIOBuilder, ShaderSet, UniformBufferObject, VertexBuffer, Window,
};
use ash::vk;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use anasaizi_core::{engine, reexports::nalgebra as math};

use anasaizi_profile::profile;

use anasaizi_core::{
    debug::{start_profiler, stop_profiler},
    engine::{image::Texture, VulkanApplication, VulkanRenderer, FRAGMENT_SHADER, VERTEX_SHADER},
    math::Vertex,
    model::{Mesh, Object},
    reexports::{
        imgui::{Context, DrawData, FontConfig, FontGlyphRanges, FontSource, TextureId},
        imgui_winit_support::{HiDpiMode, WinitPlatform},
    },
};

use anasaizi_core::{
    engine::BufferLayout,
    reexports::nalgebra::{Matrix4, Vector3},
    vulkan::structures::UIPushConstants,
};
use std::{mem, path::Path, time::Instant};
use winit::{event::MouseScrollDelta, platform::run_return::EventLoopExtRunReturn};

pub struct VulkanApp {
    vulkan_renderer: VulkanRenderer<UniformBufferObject>,
    application: VulkanApplication,

    pub viking_room_texture: [Texture; 2],

    count: f32,
}

impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let application = VulkanApplication::new("Vulkan Engine", event_loop);

        let mut vulkan_renderer = VulkanRenderer::new(&application);

        let (viking_vertices, viking_indices) = Object::load_model(Path::new("viking_room.obj"));
        let (post_vertices, post_indices) = Object::load_model(Path::new("assets/obj/post.obj"));

        let viking_mesh = Mesh::from_raw(
            &application,
            &vulkan_renderer.graphics_queue,
            &vulkan_renderer.command_pool,
            viking_vertices,
            viking_indices,
            0,
        );
        let mut post_mesh = Mesh::from_raw(
            &application,
            &vulkan_renderer.graphics_queue,
            &vulkan_renderer.command_pool,
            post_vertices,
            post_indices,
            1,
        );

        post_mesh.scale(0.01);
        post_mesh.translate(Vector3::new(100.0, 0.0, 100.0));

        let main_shader_textures = [
            Texture::create(
                &application.instance,
                &application.device,
                &vulkan_renderer.command_pool,
                &vulkan_renderer.graphics_queue,
                &Path::new("viking_room.png"),
            ),
            Texture::create(
                &application.instance,
                &application.device,
                &vulkan_renderer.command_pool,
                &vulkan_renderer.graphics_queue,
                &Path::new("texture.jpg"),
            ),
        ];

        let shader_set =
            Self::setup_main_shader(&application, &vulkan_renderer, &main_shader_textures);

        let (grid_shader, grid_mesh) = vulkan_renderer.grid_mesh(&application);
        vulkan_renderer.create_pipeline(&application, shader_set, vec![viking_mesh, post_mesh]);
        vulkan_renderer.create_pipeline(&application, grid_shader, vec![grid_mesh]);

        start_profiler();

        VulkanApp {
            vulkan_renderer,
            application,

            viking_room_texture: main_shader_textures,
            count: 0.0,
        }
    }

    pub fn setup_main_shader(
        application: &VulkanApplication,
        vulkan_renderer: &VulkanRenderer<UniformBufferObject>,
        textures: &[Texture],
    ) -> ShaderSet<UniformBufferObject> {
        let input_buffer_layout = BufferLayout::new()
            .add_float_vec3(0)
            .add_float_vec4(1)
            .add_float_vec2(2);

        let push_const_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: mem::size_of::<MeshPushConstants>() as u32,
        }];

        let mut descriptors = ShaderIOBuilder::builder()
            .add_uniform_buffer(0, vk::ShaderStageFlags::VERTEX)
            .sampler(
                1,
                vk::ShaderStageFlags::FRAGMENT,
                vulkan_renderer.texture_sampler.unwrap(),
            )
            .texture_array(
                2,
                vk::ShaderStageFlags::FRAGMENT,
                &textures,
                vulkan_renderer.texture_sampler.unwrap(),
            )
            .add_input_buffer_layout(input_buffer_layout)
            .add_push_constant_ranges(&push_const_ranges)
            .build(application, vulkan_renderer.swapchain.images.len());

        let mut builder = ShaderBuilder::builder(application, VERTEX_SHADER, FRAGMENT_SHADER, 3);
        builder.with_descriptors(descriptors);

        builder.build()
    }

    pub fn setup_ui_shader(
        application: &VulkanApplication,
        vulkan_renderer: &VulkanRenderer<UniformBufferObject>,
        imgui_context: &ImguiContext,
    ) -> ShaderSet<UniformBufferObject> {
        let input_buffer_layout = BufferLayout::new()
            .add_float_vec3(0)
            .add_float_vec4(1)
            .add_float_vec2(2);

        let push_const_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: mem::size_of::<UIPushConstants>() as u32,
        }];

        let mut descriptors = ShaderIOBuilder::builder()
            .add_static_image(
                1,
                vk::ShaderStageFlags::FRAGMENT,
                &imgui_context.ui_font_texture,
                vulkan_renderer.texture_sampler.unwrap(),
            )
            .add_input_buffer_layout(input_buffer_layout)
            .add_push_constant_ranges(&push_const_ranges)
            .build(application, vulkan_renderer.swapchain.images.len());

        let mut builder = ShaderBuilder::builder(
            application,
            "assets\\shaders\\build\\ui_vert.spv",
            "assets\\shaders\\build\\ui_frag.spv",
            3,
        );
        builder.with_descriptors(descriptors);

        builder.build()
    }

    #[profile(Sandbox)]
    fn update_uniform(&mut self, _current_image: usize) {
        self.count += 1.0 / 1000.0;
        let current_frame = self.vulkan_renderer.current_frame();

        let (view, perspective) = {
            let camera = self.vulkan_renderer.camera();
            camera.reload();
            (camera.view(), camera.projection())
        };

        for pipeline in self.vulkan_renderer.pipelines.iter_mut() {
            for mesh in pipeline.meshes.iter_mut() {
                mesh.rotate(Vector3::new(0.0, self.count, 0.0));
            }

            //if camera.is_dirty() {
            let uniform_mut = pipeline.shader.uniform_mut();
            uniform_mut.view_matrix = view;
            uniform_mut.projection_matrix = perspective;
            pipeline
                .shader
                .update_uniform(&self.application.device, current_frame);
            // }
        }
    }

    #[profile(Sandbox)]
    fn draw_frame(&mut self, draw_data: &DrawData) {
        self.update_uniform(self.vulkan_renderer.current_frame());
        self.vulkan_renderer.draw(&self.application, draw_data);
    }

    pub fn main_loop(mut self, mut event_loop: EventLoop<()>) {
        let mut run = true;

        let mut context = ImguiContext::new(
            &self.application.window,
            &self.application.device,
            &self.application.instance,
            &self.vulkan_renderer.command_pool,
            &self.vulkan_renderer.graphics_queue,
        );

        let ui_shader = Self::setup_ui_shader(&self.application, &self.vulkan_renderer, &context);
        let pipeline = Pipeline::ui_pipeline(
            &self.application.device,
            &self.vulkan_renderer.render_pass,
            ui_shader,
        );

        self.vulkan_renderer.ui_pipeline = Some(pipeline);

        while run {
            event_loop.run_return(|event, _, control_flow| {
                context.handle_event(&event, &self.application.window);

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                            run = false;
                            self.destroy();
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
                    _ => (),
                }
                *control_flow = ControlFlow::Exit;
            });

            if !run {
                break;
            }

            context.start_frame(&self.application.window);

            context.update(&self.application.window);

            let ui = context.imgui_context.frame();
            let mut opened = false;
            ui.show_demo_window(&mut opened);
            //     .text(im_str!("Hello world!"));
            // ui.text(im_str!("こんにちは世界！"));
            // ui.text(im_str!("This...is...imgui-rs!"));
            // ui.separator();
            // let mouse_pos = ui.io().mouse_pos;
            // ui.text(format!(
            //     "Mouse Position: ({:.1},{:.1})",
            //     mouse_pos[0], mouse_pos[1]
            // ));

            context
                .platform
                .prepare_render(&ui, &self.application.window);

            let draw_data = ui.render();

            self.draw_frame(draw_data)
        }
    }

    fn destroy(&self) {
        self.vulkan_renderer.destroy(&self.application.device);
    }
}

pub struct ImguiContext {
    pub platform: WinitPlatform,
    pub imgui_context: Context,
    pub ui_font_texture: Texture,
    pub last_frame: Instant,
}

impl ImguiContext {
    pub fn new(
        window: &Window,
        device: &LogicalDevice,
        instance: &Instance,
        command_pool: &CommandPool,
        submit_queue: &Queue,
    ) -> ImguiContext {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[
            FontSource::DefaultFontData {
                config: Some(FontConfig {
                    size_pixels: font_size,
                    ..FontConfig::default()
                }),
            },
            FontSource::TtfData {
                data: include_bytes!("../assets/mplus-1p-regular.ttf"),
                size_pixels: font_size,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.75,
                    glyph_ranges: FontGlyphRanges::japanese(),
                    ..FontConfig::default()
                }),
            },
        ]);
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

        // Fonts texture
        let fonts_texture = {
            let mut fonts = imgui.fonts();
            let atlas_texture = fonts.build_rgba32_texture();
            println!("{} {}", atlas_texture.width, atlas_texture.height);
            Texture::from_bytes(
                instance,
                device,
                command_pool,
                submit_queue,
                &atlas_texture.data,
                atlas_texture.width,
                atlas_texture.height,
            )
        };

        {
            let mut fonts = imgui.fonts();
            fonts.tex_id = TextureId::from(usize::MAX);
        }
        ImguiContext {
            imgui_context: imgui,
            platform,
            ui_font_texture: fonts_texture,
            last_frame: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>, window: &Window) {
        self.platform
            .handle_event(self.imgui_context.io_mut(), &window, &event);
    }

    pub fn start_frame(&mut self, window: &Window) {
        let io = self.imgui_context.io_mut();
        self.platform
            .prepare_frame(io, &window.window)
            .expect("Failed to start frame");
    }

    pub fn update(&mut self, _window: &Window) {
        let io = self.imgui_context.io_mut();
        let now = Instant::now();
        io.update_delta_time(now - self.last_frame);
        self.last_frame = now;
    }

    pub fn end_frame(&self) {}
}
