use anasaizi_core::vulkan::{Pipeline, ShaderBuilder, ShaderFlags, ShaderIOBuilder, ShaderSet};
use winit::event_loop::EventLoop;

use anasaizi_profile::profile;

use anasaizi_core::{
    debug::start_profiler,
    engine::{
        image::Texture, resources::TextureLoader, BufferLayout, GpuMeshMemory, Layer,
        LightUniformObject, MatrixUniformObject, MeshPushConstants, PBRMaps, PBRMeshPushConstants,
        RenderLayer, Transform, UIPushConstants, VulkanApplication, FRAGMENT_SHADER, VERTEX_SHADER,
    },
    libs::{
        ash::{self, vk},
        hecs::Entity,
        image::GenericImageView,
        nalgebra::{Vector3, Vector4},
    },
    model::Object,
};

use crate::{game_layer::Application, imgui_layer::ImguiLayer};

use anasaizi_core::engine::{
    gltf::{load_gltf_scene, GltfPBRShaderConstants, Root, Scene},
    GlTFPBRMeshPushConstants, LightUniformObjectGltf,
};
use std::{
    collections::HashMap,
    ffi::{c_void, CStr},
    mem,
    mem::size_of,
    path::Path,
    ptr,
    sync::Arc,
    time::{Duration, Instant},
};

pub const MAIN_MESH_PIPELINE_ID: u32 = 0;
const GRID_PIPELINE_ID: u32 = 1;
const UI_PIPELINE_ID: u32 = 2;
pub const PBR_MESH_PIPELINE_ID: u32 = 3;
pub const START_GLFT_PIPELINE_ID: u32 = 4;

const VIKING_TEXTURE_ID: i32 = 0;
const POST_TEXTURE_ID: i32 = 1;
const WINDOW_TEXTURE_ID: i32 = 2;

pub struct VulkanApp {
    vulkan_renderer: RenderLayer,
    application: VulkanApplication,

    pub textures: Vec<Texture>,

    count: f32,
    pub light_entity: Entity,

    debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
    debug_merssager: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanApp {
    async fn load_scene(
        vulkan_renderer: &mut RenderLayer,
        application: &VulkanApplication,
        texture: &[Texture],
    ) {
        let a = load_gltf_scene(
            vulkan_renderer.render_context(application),
            "E:\\programming\\Anasazi\\anasaizi-editor\\assets\\gltf\\sponza\\Sponza.gltf",
            0,
        )
        .await;
        //let a = load_gltf_scene(vulkan_renderer.render_context(application), "E:\\programming\\Anasazi\\anasaizi-editor\\assets\\gltf\\basic\\BoxMultiScene.gltf", 0).await;

        let root = a.0;
        let scene = a.1;

        let mut pipeline_id = START_GLFT_PIPELINE_ID;

        for (flag, entities) in root.entities {
            let tx = if root.textures.len() == 0 {
                texture
            } else {
                &root.textures
            };

            let mut constants = GltfPBRShaderConstants::from(flag);
            constants.texture_array_lenght = tx.len() as u32;

            let mut shader =
                Self::setup_gltf_pbr_shader(application, &vulkan_renderer, tx, constants);

            for (memory, transform, material) in entities {
                vulkan_renderer.world.spawn((
                    memory.clone(),
                    transform.clone(),
                    pipeline_id,
                    material,
                ));
            }

            vulkan_renderer.create_pipeline(application, shader, pipeline_id);

            pipeline_id += 1;
        }
    }

    pub async fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let application = VulkanApplication::new("Vulkan Engine", event_loop);

        let mut vulkan_renderer = RenderLayer::new(&application);

        let mut texture_loader =
            TextureLoader::new(Arc::from(vulkan_renderer.render_context(&application)));
        texture_loader.load_path("assets/textures/white.png", "colors.white", false);
        //
        texture_loader.load_path("assets/textures/marble/albedo.jpg", "marble.albedo", false);
        texture_loader.load_path(
            "assets/textures/marble/roughness.jpg",
            "marble.roughness",
            false,
        );
        texture_loader.load_path("assets/textures/marble/ao.jpg", "marble.ao", false);
        texture_loader.load_path(
            "assets/textures/marble/metallness.jpg",
            "marble.metallness",
            false,
        );
        texture_loader.load_path("assets/textures/marble/normal.jpg", "marble.normal", false);

        // texture_loader.load_path("assets/textures/cabin/albedo.jpg", "cabin.albedo", false);
        // texture_loader.load_path("assets/textures/cabin/roughness.jpg", "cabin.roughness", false);
        // texture_loader.load_path("assets/textures/cabin/displacement.jpg", "cabin.displacement", false);
        // texture_loader.load_path("assets/textures/cabin/normal.jpg", "cabin.normal", false);

        let textures = texture_loader.wait_loading().await;
        //
        let main_shader_textures = [
            textures.query("colors.white").owned_texture(), // 0
            textures.query("marble.albedo").owned_texture(), // 1
            textures.query("marble.roughness").owned_texture(),
            textures.query("marble.ao").owned_texture(),
            textures.query("marble.metallness").owned_texture(),
            textures.query("marble.normal").owned_texture(),
            //
            // textures.query("cabin.albedo").owned_texture(),     // 6
            // textures.query("cabin.roughness").owned_texture(),
            // textures.query("cabin.displacement").owned_texture(),
            // textures.query("cabin.normal").owned_texture(),
        ];

        Self::load_scene(&mut vulkan_renderer, &application, &main_shader_textures).await;

        let (sphere_vertices, sphere_indices) =
            Object::load_model(Path::new("assets/obj/sphere.obj"));
        //let (cabin_vertices, cabin_indices) = Object::load_model(Path::new("assets/obj/cabin.obj"));

        let render_context = vulkan_renderer.render_context(&application);
        vulkan_renderer.initialize(&application.window, &render_context);

        let light_cube_mesh_memory = GpuMeshMemory::from_raw(
            &render_context,
            sphere_vertices.clone(),
            sphere_indices.clone(),
            0,
        );

        let lighting_shader_set =
            Self::setup_pbr_shader(&application, &vulkan_renderer, &main_shader_textures);

        let main_shader_set =
            Self::setup_main_shader(&application, &vulkan_renderer, &main_shader_textures);

        let (grid_shader, grid_mesh) = vulkan_renderer.grid_mesh(&application, &render_context);

        vulkan_renderer.create_pipeline(&application, main_shader_set, MAIN_MESH_PIPELINE_ID);
        vulkan_renderer.create_pipeline(&application, lighting_shader_set, PBR_MESH_PIPELINE_ID);
        //vulkan_renderer.create_pipeline(&application, grid_shader, GRID_PIPELINE_ID);

        Self::initialize_uniform_objects(&mut vulkan_renderer);

        start_profiler();

        // for i in 0..1 {
        //     let sphere = GpuMeshMemory::from_raw(
        //         &render_context,
        //         sphere_vertices.clone(),
        //         sphere_indices.clone(),
        //         2,
        //     );
        //
        //     vulkan_renderer.world.spawn((
        //         sphere,
        //         Transform::new(1.0)
        //             .with_const_scale(0.1)
        //             .with_translate(Vector3::new(-5.0 + (i as f32) * 3.0 as f32, 2.0, 0.0)),
        //         PBR_MESH_PIPELINE_ID,
        //         PBRMaps {
        //             albedo: 1,
        //             roughness: 2,
        //             ao: 3,
        //             metalness: 4,
        //             normal: 5,
        //             displacement: -1
        //         },
        //     ));
        // }

        // let cabin = GpuMeshMemory::from_raw(
        //     &render_context,
        //     cabin_vertices.clone(),
        //     cabin_indices.clone(),
        //     2,
        // );
        //
        // vulkan_renderer.world.spawn((
        //     cabin,
        //     Transform::new(1.0)
        //         .with_const_scale(0.01)
        //         .with_translate(Vector3::new(0.0, 2.0, 0.0)),
        //     PBR_MESH_PIPELINE_ID,
        //     PBRMaps {
        //         albedo: 6,
        //         roughness: 7,
        //         ao: -1,
        //         metalness: -1,
        //         displacement: 8,
        //         normal: 9,
        //     },
        // ));

        let light_entity = vulkan_renderer.world.spawn((
            light_cube_mesh_memory,
            Transform::new(1.0)
                .with_const_scale(0.0001)
                .with_translate(Vector3::new(0.0, 3.0, -30.0)),
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
            .add_float_vec3(3)
            .add_float_vec3(4)
            .add_float_vec3(5);

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
                vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
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
            "assets\\shaders\\build\\pbr.vert.spv",
            "assets\\shaders\\build\\pbr.frag.spv",
            3,
        )
        .with_descriptors(descriptors)
        .build()
    }

    pub fn setup_gltf_pbr_shader(
        application: &VulkanApplication,
        vulkan_renderer: &RenderLayer,
        textures: &[Texture],
        mut specialisation_constant_data: GltfPBRShaderConstants,
    ) -> ShaderSet {
        let mut input_buffer_layout = BufferLayout::new()
            .add_float_vec4(0) // position
            .add_float_vec4(2) // normal
            .add_float_vec4(4) // tangent
            .add_float_vec2(6) // tex coord 0
            .add_float_vec2(7) // tex coord 1
            .add_float_vec4(8); // color

        println!(
            "size pusH: {}\n{:?}",
            size_of::<GltfPBRShaderConstants>(),
            specialisation_constant_data
        );

        let mut constant_layout = BufferLayout::new();

        for i in 0..11 {
            constant_layout = constant_layout.add_bool(i);
        }

        let specialization_constants = constant_layout.build_specialisation_constants();

        let push_const_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: mem::size_of::<GlTFPBRMeshPushConstants>() as u32,
        }];

        let mut descriptors = ShaderIOBuilder::builder()
            .add_uniform_buffer(
                0,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                &vulkan_renderer.render_context(application),
                vulkan_renderer.swapchain.images.len(),
                unsafe { size_of::<MatrixUniformObject>() },
            )
            .add_uniform_buffer(
                3,
                vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
                &vulkan_renderer.render_context(application),
                vulkan_renderer.swapchain.images.len(),
                unsafe { size_of::<LightUniformObjectGltf>() },
            )
            .add_push_constant_ranges(&push_const_ranges)
            .add_input_buffer_layout(input_buffer_layout)
            .add_specialization_constants(specialisation_constant_data, specialization_constants)
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
            );

        let shaderio = descriptors.build(
            &vulkan_renderer.render_context(application),
            vulkan_renderer.swapchain.images.len(),
        );

        ShaderBuilder::builder(
            application,
            "assets\\shaders\\build\\pbr_gltf.vert.spv",
            "assets\\shaders\\build\\pbr_gltf.frag.spv",
            3,
        )
        .with_descriptors(shaderio)
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
            .add_float_vec3(3)
            .add_float_vec3(4)
            .add_float_vec3(5);

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
            .add_float_vec3(3)
            .add_float_vec3(4)
            .add_float_vec3(5);

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
            "assets\\shaders\\build\\ui_vert.vert.spv",
            "assets\\shaders\\build\\ui_frag.frag.spv",
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

            if pipeline.pipeline_id() >= START_GLFT_PIPELINE_ID
                && pipeline.pipeline_id() <= START_GLFT_PIPELINE_ID + 100
            {
                let mut light = LightUniformObjectGltf::default();
                light.ambient_color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                light.ambient_light_intensity = 0.2;
                light.light_direction = Vector4::new(0.0, 0.5, 0.5, 1.0);
                light.light_color = Vector4::new(5.0, 5.0, 5.0, 1.0);

                let mut gltf = GlTFPBRMeshPushConstants::default();

                pipeline.shader.add_uniform_object(light);
                pipeline.shader.add_uniform_object(gltf);
            }
        }
    }

    #[profile(Sandbox)]
    fn update_uniform(
        vulkan_renderer: &mut RenderLayer,
        application: &VulkanApplication,
        _imgui_layer: &ImguiLayer,
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

            if pipeline.pipeline_id() >= START_GLFT_PIPELINE_ID
                && pipeline.pipeline_id() <= START_GLFT_PIPELINE_ID + 5
            {
                let light_entity = vulkan_renderer
                    .world
                    .get::<Transform>(light_entity)
                    .unwrap();
                let light_pos = light_entity.translate_factor();
                let camera_pos = vulkan_renderer.camera.position();

                pipeline.shader.update_uniform::<LightUniformObjectGltf>(
                    &application.device,
                    vulkan_renderer.current_frame,
                    1,
                    &move |obj| {
                        obj.view_pos =
                            Vector4::new(camera_pos[0], camera_pos[1], camera_pos[2], 1.0);
                        obj.position = Vector4::new(light_pos[0], light_pos[1], light_pos[2], 1.0);
                        obj.light_color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                        obj.light_direction = Vector4::new(0.0, 0.5, 0.5, 1.0);
                        obj.ambient_light_intensity = 0.2;
                        obj.ambient_color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                    },
                );
            }
            if pipeline.pipeline_id() == PBR_MESH_PIPELINE_ID {
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

    pub fn main_loop(self, mut event_loop: EventLoop<()>) {
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
