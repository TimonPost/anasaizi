use anasaizi_core::vulkan::{Pipeline, ShaderBuilder, ShaderIOBuilder, ShaderSet};
use winit::event_loop::EventLoop;

use anasaizi_profile::profile;

use anasaizi_core::{
    debug::start_profiler,
    engine::{image::Texture, RenderLayer, VulkanApplication, FRAGMENT_SHADER, VERTEX_SHADER},
    libs::ash::{self, vk},
    model::Object,
};

use crate::{game_layer::Application, imgui_layer::ImguiLayer};
use anasaizi_core::{
    engine::{BufferLayout, GpuMeshMemory, Layer, Transform, UIPushConstants},
    libs::{hecs::Entity, nalgebra::Vector3},
};

use anasaizi_core::{
    engine::{
        LightUniformObject, MaterialUniformObject, MatrixUniformObject, MeshPushConstants, PBRMaps,
        PBRMeshPushConstants, RenderContext, World,
    },
    libs::nalgebra::{Vector, Vector4},
};

use std::{
    collections::HashMap,
    ffi::{c_void, CStr},
    mem,
    mem::size_of,
    path::Path,
    ptr,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anasaizi_core::{
    engine::resources::TextureLoader,
    libs::{image, image::GenericImageView},
};
use std::path::PathBuf;

pub const MAIN_MESH_PIPELINE_ID: u32 = 0;
const GRID_PIPELINE_ID: u32 = 1;
const UI_PIPELINE_ID: u32 = 2;
pub const PBR_MESH_PIPELINE_ID: u32 = 3;

const VIKING_TEXTURE_ID: i32 = 0;
const POST_TEXTURE_ID: i32 = 1;
const WINDOW_TEXTURE_ID: i32 = 2;

pub struct VulkanApp {
    vulkan_renderer: RenderLayer,
    application: VulkanApplication,

    pub textures: Vec<Texture>,

    count: f32,
    pub light_entity: Entity,
    // pub post_entity: Entity,
    //pub grid_entity: Entity,
    debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
    debug_merssager: Option<vk::DebugUtilsMessengerEXT>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Material {
    Emerald,
    Jade,
    Obsidian,
    Pearl,
    Ruby,
    Turquoise,
    Brass,
    Bronze,
    Chrome,
}

impl Material {
    pub fn get_vectors(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>, f32) {
        if self == &Material::Emerald {
            (
                Vector3::new(0.0215, 0.1745, 0.0215),
                Vector3::new(0.07568, 0.61424, 0.07568),
                Vector3::new(0.633, 0.727811, 0.633),
                0.6,
            )
        } else if self == &Material::Jade {
            (
                Vector3::new(0.135, 0.2225, 0.1575),
                Vector3::new(0.54, 0.89, 0.63),
                Vector3::new(0.316228, 0.316228, 0.316228),
                0.1,
            )
        } else if self == &Material::Obsidian {
            (
                Vector3::new(0.05375, 0.05, 0.0662),
                Vector3::new(0.18275, 0.17, 0.22525),
                Vector3::new(0.332741, 0.328634, 0.346435),
                0.3,
            )
        } else if self == &Material::Pearl {
            (
                Vector3::new(0.25, 0.20725, 0.2072),
                Vector3::new(1.0, 0.829, 0.829),
                Vector3::new(0.296648, 0.296648, 0.296648),
                0.0088,
            )
        } else if self == &Material::Ruby {
            (
                Vector3::new(0.1745, 0.01175, 0.0117),
                Vector3::new(0.61424, 0.04136, 0.04136),
                Vector3::new(0.727811, 0.626959, 0.626959),
                0.6,
            )
        } else if self == &Material::Turquoise {
            (
                Vector3::new(0.1, 0.18725, 0.1745),
                Vector3::new(0.396, 0.74151, 0.69102),
                Vector3::new(0.297254, 0.30829, 0.306678),
                0.1,
            )
        } else if self == &Material::Brass {
            (
                Vector3::new(0.329412, 0.223529, 0.027451),
                Vector3::new(0.780392, 0.568627, 0.113725),
                Vector3::new(0.992157, 0.941176, 0.807843),
                0.21794872,
            )
        } else if self == &Material::Bronze {
            (
                Vector3::new(0.2125, 0.1275, 0.054),
                Vector3::new(0.714, 0.4284, 0.18144),
                Vector3::new(0.393548, 0.271906, 0.166721),
                0.2,
            )
        } else if self == &Material::Chrome {
            (
                Vector3::new(0.25, 0.25, 0.25),
                Vector3::new(0.4, 0.4, 0.4),
                Vector3::new(0.774597, 0.774597, 0.774597),
                0.6,
            )
        } else {
            panic!("No such material supported.")
        }
    }
}

impl VulkanApp {
    pub async fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let application = VulkanApplication::new("Vulkan Engine", event_loop);

        let mut vulkan_renderer = RenderLayer::new(&application);

        let (cube_vertices, cube_indices) = Object::load_model(Path::new("assets/obj/cube.obj"));

        let render_context = vulkan_renderer.render_context(&application);
        vulkan_renderer.initialize(&application.window, &render_context);

        let mut texture_loader = TextureLoader::new(vulkan_renderer.render_context(&application));
        texture_loader.load("assets/textures/white.png", "colors.white", false);

        texture_loader.load("assets/textures/marble/albedo.jpg", "marble.albedo", false);
        texture_loader.load(
            "assets/textures/marble/roughness.jpg",
            "marble.roughness",
            false,
        );
        texture_loader.load("assets/textures/marble/ao.jpg", "marble.ao", false);
        texture_loader.load(
            "assets/textures/marble/metallness.jpg",
            "marble.metallness",
            false,
        );
        texture_loader.load("assets/textures/marble/normal.jpg", "marble.normal", false);

        let textures = texture_loader.wait_loading().await;

        let main_shader_textures = [
            textures.query("colors.white").owned_texture(),
            textures.query("marble.albedo").owned_texture(),
            textures.query("marble.roughness").owned_texture(),
            textures.query("marble.ao").owned_texture(),
            textures.query("marble.metallness").owned_texture(),
            textures.query("marble.normal").owned_texture(),
        ];

        let cube_mesh_memory = GpuMeshMemory::from_raw(
            &render_context,
            cube_vertices.clone(),
            cube_indices.clone(),
            2,
        );

        let light_cube_mesh_memory =
            GpuMeshMemory::from_raw(&render_context, cube_vertices, cube_indices, 0);

        let lighting_shader_set =
            Self::setup_pbr_shader(&application, &vulkan_renderer, &main_shader_textures);

        let main_shader_set =
            Self::setup_main_shader(&application, &vulkan_renderer, &main_shader_textures);

        let (grid_shader, grid_mesh) = vulkan_renderer.grid_mesh(&application, &render_context);

        vulkan_renderer.create_pipeline(&application, main_shader_set, MAIN_MESH_PIPELINE_ID);
        vulkan_renderer.create_pipeline(&application, lighting_shader_set, PBR_MESH_PIPELINE_ID);
        vulkan_renderer.create_pipeline(&application, grid_shader, GRID_PIPELINE_ID);

        Self::initialize_uniform_objects(&mut vulkan_renderer);

        start_profiler();

        vulkan_renderer.world.spawn((
            cube_mesh_memory,
            Transform::new(1.0)
                .with_scale(0.2)
                .with_translate(Vector3::new(-1.0, 2.0, 3.0)),
            PBR_MESH_PIPELINE_ID,
            PBRMaps {
                albedo: 1,
                roughness: 2,
                ao: 3,
                metalness: 4,
                normal: 5,
            },
        ));

        let light_entity = vulkan_renderer.world.spawn((
            light_cube_mesh_memory,
            Transform::new(1.0)
                .with_scale(0.01)
                .with_translate(Vector3::new(0.0, 3.0, -3.0)),
            MAIN_MESH_PIPELINE_ID,
        ));

        vulkan_renderer
            .world
            .spawn((grid_mesh, Transform::new(1.0), GRID_PIPELINE_ID));

        let (debug_utils_loader, debug_merssager) =
            setup_debug_utils(true, &application.instance.entry(), &application.instance);

        VulkanApp {
            vulkan_renderer,
            application,
            textures: Vec::from(main_shader_textures),
            count: 0.0,

            debug_merssager: Some(debug_merssager),
            debug_utils_loader: Some(debug_utils_loader),
            light_entity,
        }
    }

    pub fn setup_pbr_shader(
        application: &VulkanApplication,
        vulkan_renderer: &RenderLayer,
        textures: &[Texture],
    ) -> ShaderSet {
        let input_buffer_layout = BufferLayout::new()
            .add_float_vec3(0)
            .add_float_vec4(1)
            .add_float_vec2(2)
            .add_float_vec3(3);

        let push_const_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: mem::size_of::<PBRMeshPushConstants>() as u32,
        }];

        let descriptors = ShaderIOBuilder::builder()
            .add_uniform_buffer(
                0,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                &vulkan_renderer.render_context(application),
                vulkan_renderer.swapchain.images.len(),
                unsafe { size_of::<MatrixUniformObject>() },
            )
            .add_uniform_buffer(
                3,
                vk::ShaderStageFlags::FRAGMENT,
                &vulkan_renderer.render_context(application),
                vulkan_renderer.swapchain.images.len(),
                unsafe { size_of::<LightUniformObject>() },
            )
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

        ShaderBuilder::builder(
            application,
            "assets\\shaders\\build\\pong_lighting_vert.spv",
            "assets\\shaders\\build\\pong_lighting_frag.spv",
            3,
        )
        .with_descriptors(descriptors)
        .build()
    }

    pub fn setup_main_shader(
        application: &VulkanApplication,
        vulkan_renderer: &RenderLayer,
        textures: &[Texture],
    ) -> ShaderSet {
        let input_buffer_layout = BufferLayout::new()
            .add_float_vec3(0)
            .add_float_vec4(1)
            .add_float_vec2(2)
            .add_float_vec3(3);

        let push_const_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: mem::size_of::<MeshPushConstants>() as u32,
        }];

        let descriptors = ShaderIOBuilder::builder()
            .add_uniform_buffer(
                0,
                vk::ShaderStageFlags::VERTEX,
                &vulkan_renderer.render_context(application),
                vulkan_renderer.swapchain.images.len(),
                unsafe { size_of::<MatrixUniformObject>() },
            )
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
        vulkan_renderer: &RenderLayer,
        texture: &Texture,
    ) -> ShaderSet {
        let input_buffer_layout = BufferLayout::new()
            .add_float_vec3(0)
            .add_float_vec4(1)
            .add_float_vec2(2)
            .add_float_vec3(3);

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
    fn initialize_uniform_objects(vulkan_renderer: &mut RenderLayer) {
        let (view, perspective) = {
            let camera = vulkan_renderer.camera();
            camera.reload();
            (camera.view(), camera.projection())
        };

        for pipeline in vulkan_renderer.pipelines.iter_mut() {
            pipeline.shader.add_uniform_object(MatrixUniformObject {
                projection_matrix: perspective,
                view_matrix: view,
            });

            if pipeline.pipeline_id() == PBR_MESH_PIPELINE_ID {
                pipeline
                    .shader
                    .add_uniform_object(LightUniformObject::default());
            }
        }
    }

    #[profile(Sandbox)]
    fn update_uniform(
        material: Material,
        vulkan_renderer: &mut RenderLayer,
        application: &VulkanApplication,
        imgui_layer: &ImguiLayer,
        light_entity: Entity,
    ) {
        let (view, perspective) = {
            let camera = vulkan_renderer.camera();
            camera.reload();
            (camera.view(), camera.projection())
        };

        for pipeline in vulkan_renderer.pipelines.iter_mut() {
            pipeline.shader.update_uniform::<MatrixUniformObject>(
                &application.device,
                vulkan_renderer.current_frame,
                0,
                &move |obj| {
                    obj.view_matrix = view.clone();
                    obj.projection_matrix = perspective.clone();
                },
            );

            if pipeline.pipeline_id() == PBR_MESH_PIPELINE_ID {
                // pipeline.shader.update_uniform::<MaterialUniformObject>(&application.device, vulkan_renderer.current_frame, 1, &move |obj| {
                //     obj.specular_texture_id = 3;
                //     obj.shininess = 64.0;
                //     obj.diffuse_texture_id = 2;
                // });

                let light_entity = vulkan_renderer
                    .world
                    .get::<Transform>(light_entity)
                    .unwrap();
                let light_pos = light_entity.translate_factor();
                let camera_pos = vulkan_renderer.camera.position();

                pipeline.shader.update_uniform::<LightUniformObject>(
                    &application.device,
                    vulkan_renderer.current_frame,
                    1,
                    &move |obj| {
                        obj.view_pos =
                            Vector4::new(camera_pos[0], camera_pos[1], camera_pos[2], 1.0);
                        obj.position = Vector4::new(light_pos[0], light_pos[1], light_pos[2], 1.0);
                        obj.light_color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                    },
                );
            }
        }
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
        let pipeline = Pipeline::ui_pipeline(
            &application.device,
            &vulkan_renderer.render_pass,
            ui_shader,
            UI_PIPELINE_ID,
        );

        vulkan_renderer.ui_pipeline = Some(pipeline);

        let mut render_layers = vec![vulkan_renderer];

        let mut ui_layers = vec![ui_layer];

        let mut rand_counter = Instant::now();
        let materials = vec![
            Material::Emerald,
            Material::Jade,
            Material::Obsidian,
            Material::Pearl,
            Material::Ruby,
            Material::Turquoise,
            Material::Brass,
            Material::Bronze,
            Material::Chrome,
        ];
        let mut active_material = materials[0];
        let mut next_index = 0;

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

            if rand_counter.elapsed() >= Duration::from_millis(2000) {
                rand_counter = Instant::now();
                active_material = materials[next_index];

                if next_index == materials.len() - 1 {
                    next_index = 0;
                } else {
                    next_index += 1;
                }
            }

            Self::update_uniform(
                active_material,
                &mut render_layers[0],
                &application,
                &ui_layers[0],
                self.light_entity,
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
