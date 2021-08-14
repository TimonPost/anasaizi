use anasaizi_core::vulkan::{
    MeshPushConstants, Pipeline,
    ShaderBuilder, ShaderIOBuilder, ShaderSet, UniformBufferObject,
};
use ash::vk;
use winit::{
    event_loop::{EventLoop},
};



use anasaizi_profile::profile;

use anasaizi_core::{
    debug::{start_profiler},
    engine::{image::Texture, RenderLayer, VulkanApplication, FRAGMENT_SHADER, VERTEX_SHADER},
    model::{Mesh, Object},
};

use crate::{game_layer::GameLayer, imgui_layer::ImguiLayer};
use anasaizi_core::{
    engine::{BufferLayout, Layer},
    reexports::{
        nalgebra::{Vector3},
    },
    vulkan::structures::UIPushConstants,
};
use std::{
    mem,
    path::Path,
};


pub struct VulkanApp {
    vulkan_renderer: RenderLayer<UniformBufferObject>,
    application: VulkanApplication,

    pub viking_room_texture: [Texture; 2],

    count: f32,
}

impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let application = VulkanApplication::new("Vulkan Engine", event_loop);

        let mut vulkan_renderer = RenderLayer::new(&application);

        let (viking_vertices, viking_indices) = Object::load_model(Path::new("viking_room.obj"));
        let (post_vertices, post_indices) = Object::load_model(Path::new("assets/obj/post.obj"));

        let render_context = vulkan_renderer.render_context(&application);
        vulkan_renderer.initialize(&application.window, &render_context);

        let viking_mesh = Mesh::from_raw(&render_context, viking_vertices, viking_indices, 0);
        let mut post_mesh = Mesh::from_raw(&render_context, post_vertices, post_indices, 1);

        post_mesh.scale(0.01);
        post_mesh.translate(Vector3::new(100.0, 0.0, 100.0));

        let main_shader_textures = [
            Texture::create(&render_context, &Path::new("viking_room.png")),
            Texture::create(&render_context, &Path::new("texture.jpg")),
        ];

        let shader_set =
            Self::setup_main_shader(&application, &vulkan_renderer, &main_shader_textures);

        let (grid_shader, grid_mesh) = vulkan_renderer.grid_mesh(&application, &render_context);
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
        vulkan_renderer: &RenderLayer<UniformBufferObject>,
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

        let descriptors = ShaderIOBuilder::builder()
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
            .build(
                &vulkan_renderer.render_context(application),
                vulkan_renderer.swapchain.images.len(),
            );

        let mut builder = ShaderBuilder::builder(application, VERTEX_SHADER, FRAGMENT_SHADER, 3);
        builder.with_descriptors(descriptors);

        builder.build()
    }

    pub fn setup_ui_shader(
        application: &VulkanApplication,
        vulkan_renderer: &RenderLayer<UniformBufferObject>,
        texture: &Texture,
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

        let descriptors = ShaderIOBuilder::builder()
            .add_static_image(
                1,
                vk::ShaderStageFlags::FRAGMENT,
                &texture,
                vulkan_renderer.texture_sampler.unwrap(),
            )
            .add_input_buffer_layout(input_buffer_layout)
            .add_push_constant_ranges(&push_const_ranges)
            .build(
                &vulkan_renderer.render_context(application),
                vulkan_renderer.swapchain.images.len(),
            );

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
    fn update_uniform(
        vulkan_renderer: &mut RenderLayer<UniformBufferObject>,
        ui_layer: &ImguiLayer,
        count: &mut f32,
        application: &VulkanApplication,
    ) {
        *count += 1.0 / 1000.0;
        let current_frame = vulkan_renderer.current_frame();

        let (view, perspective) = {
            let camera = vulkan_renderer.camera();
            camera.reload();
            (camera.view(), camera.projection())
        };

        for pipeline in vulkan_renderer.pipelines.iter_mut() {
            let mut mesh = &mut pipeline.meshes[0];
            let rotate = ui_layer.data.object_rotate;
            let translate = ui_layer.data.object_translate;
            let scale = ui_layer.data.object_scale;

            mesh.rotate(Vector3::new(rotate[0],rotate[1],rotate[2]));
            mesh.translate(Vector3::new(translate[0],translate[1],translate[2]));
            mesh.scale(scale[0]);


            //if camera.is_dirty() {
            let uniform_mut = pipeline.shader.uniform_mut();
            uniform_mut.view_matrix = view;
            uniform_mut.projection_matrix = perspective;
            pipeline
                .shader
                .update_uniform(&application.device, current_frame);
            // }
        }
    }

    pub fn main_loop(mut self, mut event_loop: EventLoop<()>) {
        let mut application = self.application;
        let mut vulkan_renderer = self.vulkan_renderer;
        let render_context = vulkan_renderer.render_context(&application);
        let mut game_layer = GameLayer::new();

        let mut ui_layer = ImguiLayer::new(&mut application, &mut vulkan_renderer);
        ui_layer.initialize(&application.window, &render_context);
        let ui_shader =
            Self::setup_ui_shader(&application, &vulkan_renderer, &ui_layer.ui_font_texture);
        let pipeline =
            Pipeline::ui_pipeline(&application.device, &vulkan_renderer.render_pass, ui_shader);

        vulkan_renderer.ui_pipeline = Some(pipeline);

        let mut render_layers = vec![vulkan_renderer];

        let mut ui_layers = vec![ui_layer];

        let mut game_runs = true;
        while game_runs {
            game_runs = game_layer.tick(&mut event_loop);

            if !game_runs {
                break;
            }

            game_layer.before_frame();
            game_layer.run_layers(&mut render_layers, &render_context, &application);
            game_layer.run_layers(&mut ui_layers, &render_context, &application);
            game_layer.after_frame();

            Self::update_uniform(&mut render_layers[0], &ui_layers[0],&mut self.count, &application);

            render_layers[0].ui_data = ui_layers[0].draw_data;
            render_layers[0].ui_mesh = ui_layers[0].ui_mesh.as_ref().unwrap();
        }
    }

    fn destroy(&self) {
        self.vulkan_renderer.destroy(&self.application.device);
    }
}
