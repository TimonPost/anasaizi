use anasaizi_core::vulkan::{
    MeshPushConstants, Pipeline,
    ShaderBuilder, ShaderIOBuilder, ShaderSet, UniformBufferObject,
};
use ash::vk;
use winit::event_loop::EventLoop;

use anasaizi_profile::profile;

use anasaizi_core::{
    debug::start_profiler,
    engine::{image::Texture, RenderLayer, VulkanApplication, FRAGMENT_SHADER, VERTEX_SHADER},
    model::Object,
};

use crate::{game_layer::Application, imgui_layer::ImguiLayer};
use anasaizi_core::{
    engine::{BufferLayout, GpuMeshMemory, Layer, Transform},
    reexports::nalgebra::{Vector3},
    vulkan::structures::{UIPushConstants},
};

use hecs::{Entity};
use std::{
    ffi::{c_void, CStr},
    mem,
    path::Path,
    ptr,
};

pub const MAIN_MESH_PIPELINE_ID: u32 = 0;
const GRID_PIPELINE_ID: u32 = 1;
const UI_PIPELINE_ID: u32 = 2;

const VIKING_TEXTURE_ID: i32 = 0;
const POST_TEXTURE_ID: i32 = 1;
const WINDOW_TEXTURE_ID: i32 = 2;

pub struct VulkanApp {
    vulkan_renderer: RenderLayer<UniformBufferObject>,
    application: VulkanApplication,

    pub viking_room_texture: [Texture; 3],

    count: f32,
    // pub viking_entity: Entity,
    // pub post_entity: Entity,
    //pub grid_entity: Entity,
    // debug_utils_loader: ash::extensions::ext::DebugUtils,
    // debug_merssager: vk::DebugUtilsMessengerEXT,
}

impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let application = VulkanApplication::new("Vulkan Engine", event_loop);

        let mut vulkan_renderer = RenderLayer::new(&application);

        let (viking_vertices, viking_indices) = Object::load_model(Path::new("assets/obj/viking_room.obj"));
        let (post_vertices, post_indices) = Object::load_model(Path::new("assets/obj/post.obj"));
        let (window_vertices, window_indices) = Object::load_model(Path::new("assets/obj/window.obj"));
        //let (window_vertices, window_indices) = Object::load_model(Path::new("assets/obj/window.obj"));

        let render_context = vulkan_renderer.render_context(&application);
        vulkan_renderer.initialize(&application.window, &render_context);

        let main_shader_textures = [
            Texture::create(&render_context, &Path::new("assets/textures/viking_room.png")),
            Texture::create(&render_context, &Path::new("assets/textures/texture.jpg")),
            Texture::create(&render_context, &Path::new("assets/textures/window.jpg")),
        ];

        let viking_mesh_memory = GpuMeshMemory::from_raw(
            &render_context,
            viking_vertices,
            viking_indices,
            VIKING_TEXTURE_ID,
        );
        let post_mesh_memory = GpuMeshMemory::from_raw(
            &render_context,
            post_vertices,
            post_indices,
            POST_TEXTURE_ID,
        );
        let window_mesh_memory = GpuMeshMemory::from_raw(
            &render_context,
            window_vertices,
            window_indices,
            WINDOW_TEXTURE_ID,
        );

        let shader_set =
            Self::setup_main_shader(&application, &vulkan_renderer, &main_shader_textures);

        let (grid_shader, grid_mesh) = vulkan_renderer.grid_mesh(&application, &render_context);

        vulkan_renderer.create_pipeline(&application, shader_set, MAIN_MESH_PIPELINE_ID);
        vulkan_renderer.create_pipeline(&application, grid_shader, GRID_PIPELINE_ID);

        start_profiler();

        let viking_entity = vulkan_renderer.world.spawn((
            viking_mesh_memory,
            Transform::new(1.0),
            MAIN_MESH_PIPELINE_ID,
        ));

        let _post_entity = vulkan_renderer.world.spawn((
            post_mesh_memory,
            Transform::new(0.01)
                .with_scale(0.01)
                .with_translate(Vector3::new(100.0, 0.0, 100.0)),
            MAIN_MESH_PIPELINE_ID,
        ));

        let _window_entity = vulkan_renderer.world.spawn((
            window_mesh_memory,
            Transform::new(0.01)
                .with_scale(0.01)
                .with_translate(Vector3::new(100.0, 0.0, 100.0)),
            MAIN_MESH_PIPELINE_ID,
        ));

        let _grid_entity =
            vulkan_renderer
                .world
                .spawn((grid_mesh, Transform::new(1.0), GRID_PIPELINE_ID));

        //let (debug_utils_loader, debug_merssager) =
        //    setup_debug_utils(true, &application.instance.entry(), &application.instance);

        VulkanApp {
            vulkan_renderer,
            application,
            viking_room_texture: main_shader_textures,
            count: 0.0,

           // viking_entity: viking_entity,
            // post_entity: viking_entity,
            // grid_entity: viking_entity,
            // debug_merssager,
            // debug_utils_loader
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

        ShaderBuilder::builder(application, VERTEX_SHADER, FRAGMENT_SHADER, 3)
            .with_descriptors(descriptors)
            .build()
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

        ShaderBuilder::builder(
            application,
            "assets\\shaders\\build\\ui_vert.spv",
            "assets\\shaders\\build\\ui_frag.spv",
            3,
        )
        .with_descriptors(descriptors)
        .build()
    }

    #[profile(Sandbox)]
    fn update_uniform(
        vulkan_renderer: &mut RenderLayer<UniformBufferObject>,
        _ui_layer: &ImguiLayer,
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
            //if camera.is_dirty() {
            let uniform_mut = pipeline.shader.uniform_mut();
            uniform_mut.view_matrix = view;
            uniform_mut.projection_matrix = perspective;

            pipeline
                .shader
                .update_uniform(&application.device, current_frame);
            // }
        }

        let uniform_mut = vulkan_renderer.object_picker.pipeline.shader.uniform_mut();
        uniform_mut.view_matrix = view;
        uniform_mut.projection_matrix = perspective;
        vulkan_renderer
            .object_picker
            .pipeline
            .shader
            .update_uniform(&application.device, 0);
    }

    pub fn main_loop(mut self, mut event_loop: EventLoop<()>) {
        let mut application = self.application;
        let mut vulkan_renderer = self.vulkan_renderer;
        let render_context = vulkan_renderer.render_context(&application);
        let mut game_layer = Application::new();

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

            Self::update_uniform(
                &mut render_layers[0],
                &ui_layers[0],
                &mut self.count,
                &application,
            );

            render_layers[0].ui_data = ui_layers[0].draw_data;
            render_layers[0].ui_mesh = ui_layers[0].ui_mesh.as_ref().unwrap();
        }
    }

    fn destroy(&self) {
        self.vulkan_renderer.destroy(&self.application.device);
    }
}

pub fn setup_debug_utils(
    is_enable_debug: bool,
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

    if is_enable_debug == false {
        (debug_utils_loader, ash::vk::DebugUtilsMessengerEXT::null())
    } else {
        let messenger_ci = populate_debug_messenger_create_info();

        let utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&messenger_ci, None)
                .expect("Debug Utils Callback")
        };

        (debug_utils_loader, utils_messenger)
    }
}

pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut(),
    }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}
