use crate::sandbox::MAIN_MESH_PIPELINE_ID;
use anasaizi_core::{
    engine::{
        image::Texture, Event, GpuMeshMemory, Layer, RenderContext, RenderLayer, Transform,
        VulkanApplication, World,
    },
    reexports::{
        imgui::{
            im_str, Context, DrawData, FontConfig, FontGlyphRanges, FontSource, ImStr, Slider,
            TextureId,
        },
        imgui_winit_support::{HiDpiMode, WinitPlatform},
        nalgebra::Vector3,
    },
    vulkan::{UniformBufferObject, Window},
};
use hecs::Entity;
use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
    mem,
    time::Duration,
};
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};

pub struct TransformInput {
    pub object_translate_x: f32,
    pub object_translate_y: f32,
    pub object_translate_z: f32,

    pub object_rotate_x: f32,
    pub object_rotate_y: f32,
    pub object_rotate_z: f32,
    pub object_scale: f32,
}

pub struct ImguiLayer {
    pub platform: WinitPlatform,
    pub imgui_context: Context,
    pub ui_font_texture: Texture,

    pub ui_mesh: Option<GpuMeshMemory>,

    pub window: *mut Window,
    pub draw_data: *const DrawData,
    world: *mut World,

    pub transform_input: TransformInput,
    pub selected_entity: Option<Entity>,
}

impl ImguiLayer {
    pub fn new(
        application: &mut VulkanApplication,
        _vulkan_renderer: &mut RenderLayer<UniformBufferObject>,
    ) -> ImguiLayer {
        let mut imgui = Context::create();

        let platform = WinitPlatform::init(&mut imgui);

        ImguiLayer {
            imgui_context: imgui,
            platform,
            ui_font_texture: unsafe { mem::zeroed() },
            ui_mesh: None,
            window: &mut application.window,
            draw_data: std::ptr::null(),
            world: &mut _vulkan_renderer.world,
            transform_input: TransformInput {
                object_scale: 0.0,

                object_translate_x: 0.0,
                object_translate_y: 0.0,
                object_translate_z: 0.0,

                object_rotate_x: 0.0,
                object_rotate_y: 0.0,
                object_rotate_z: 0.0,
            },
            selected_entity: None,
        }
    }
}

impl ImguiLayer {
    pub fn window(&mut self) -> &mut winit::window::Window {
        unsafe { &mut (*self.window).window }
    }
}

impl Layer for ImguiLayer {
    fn initialize(&mut self, window: &Window, render_context: &RenderContext) {
        self.imgui_context.set_ini_filename(None);

        let hidpi_factor = self.platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        self.imgui_context.fonts().add_font(&[
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

        self.imgui_context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        self.platform
            .attach_window(self.imgui_context.io_mut(), &window, HiDpiMode::Default);

        // Fonts texture
        let fonts_texture = {
            let mut fonts = self.imgui_context.fonts();
            let atlas_texture = fonts.build_rgba32_texture();
            println!("{} {}", atlas_texture.width, atlas_texture.height);
            Texture::from_bytes(
                render_context,
                &atlas_texture.data,
                atlas_texture.width,
                atlas_texture.height,
            )
        };

        {
            let mut fonts = self.imgui_context.fonts();
            fonts.tex_id = TextureId::from(usize::MAX);
        }

        self.ui_font_texture = fonts_texture;
    }

    fn on_event(&mut self, event: &Event) {
        match event {
            Event::Raw(event) => unsafe {
                self.platform.handle_event(
                    self.imgui_context.io_mut(),
                    unsafe { &mut (*self.window).window },
                    event,
                );

                let mut index = None;
                match event.clone().as_ref() {
                    winit::event::Event::WindowEvent { event, .. } => match event {
                        WindowEvent::KeyboardInput { input, .. } => {
                            match (input.virtual_keycode, input.state) {
                                (Some(VirtualKeyCode::Numpad1), ElementState::Pressed) => {
                                    index = Some(0)
                                }
                                (Some(VirtualKeyCode::Numpad2), ElementState::Pressed) => {
                                    index = Some(1)
                                }
                                (Some(VirtualKeyCode::Numpad3), ElementState::Pressed) => {
                                    index = Some(2)
                                }
                                (Some(VirtualKeyCode::Numpad4), ElementState::Pressed) => {
                                    index = Some(3)
                                }
                                (Some(VirtualKeyCode::Numpad5), ElementState::Pressed) => {
                                    index = Some(4)
                                }
                                _ => {}
                            };
                        }
                        _ => {}
                    },
                    _ => {}
                }

                if let Some(selected_entity) = index {
                    for (id, (transform)) in (*self.world).query::<(&Transform)>().iter() {
                        if id.id() == selected_entity {
                            self.selected_entity = Some(id);
                            let translate = transform.translate_factor();
                            let rotate = transform.rotation_factor();

                            self.transform_input.object_scale = transform.scale_factor();
                            self.transform_input.object_rotate_x = rotate[0];
                            self.transform_input.object_rotate_y = rotate[1];
                            self.transform_input.object_rotate_z = rotate[2];
                            self.transform_input.object_translate_x = translate[0];
                            self.transform_input.object_translate_y = translate[1];
                            self.transform_input.object_translate_z = translate[2];
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn start_frame(&mut self) {
        let io = self.imgui_context.io_mut();
        self.platform
            .prepare_frame(io, unsafe { &mut (*self.window).window })
            .expect("Failed to start frame");
    }

    fn on_update(
        &mut self,
        delta_time: u128,
        _render_context: &RenderContext,
        _application: &VulkanApplication,
    ) {
        let io = self.imgui_context.io_mut();
        io.update_delta_time(Duration::from_millis(delta_time as u64));

        let ui = self.imgui_context.frame();

        unsafe {
            ui.text(im_str!("Object Properties"));
        }

        if let Some(entity_id) = self.selected_entity {
            unsafe {
                for (id, (transform)) in (*self.world).query_mut::<(&mut Transform)>() {
                    if id == entity_id {
                        transform.translate(Vector3::new(
                            self.transform_input.object_translate_x,
                            self.transform_input.object_translate_y,
                            self.transform_input.object_translate_z,
                        ));
                        transform.rotate(Vector3::new(
                            self.transform_input.object_rotate_x,
                            self.transform_input.object_rotate_y,
                            self.transform_input.object_rotate_z,
                        ));
                        transform.scale(self.transform_input.object_scale as f32);

                        unsafe {
                            ui.columns(3, im_str!("Translate"), true);

                            Slider::new(im_str!("X##1"))
                                .range(-15.0..=15.0)
                                .build(&ui, &mut self.transform_input.object_translate_x);
                            ui.next_column();
                            Slider::new(im_str!("Y##1"))
                                .range(-15.0..=15.0)
                                .build(&ui, &mut self.transform_input.object_translate_y);
                            ui.next_column();
                            Slider::new(im_str!("Z##1"))
                                .range(-15.0..=15.0)
                                .build(&ui, &mut self.transform_input.object_translate_z);
                            ui.separator();
                            ui.next_column();

                            ui.columns(3, im_str!("Rotate"), true);

                            Slider::new(im_str!("X##2"))
                                .range(0.0..=PI * 2.0)
                                .build(&ui, &mut self.transform_input.object_rotate_x);
                            ui.next_column();
                            Slider::new(im_str!("Y##2"))
                                .range(0.0..=PI * 2.0)
                                .build(&ui, &mut self.transform_input.object_rotate_y);
                            ui.next_column();
                            Slider::new(im_str!("Z##2"))
                                .range(0.0..=PI * 2.0)
                                .build(&ui, &mut self.transform_input.object_rotate_z);
                            ui.separator();
                            ui.next_column();
                            Slider::new(im_str!("Scale"))
                                .range(transform.unit_scale())
                                .build(&ui, &mut self.transform_input.object_scale);
                        }
                    }
                }
            }
        }

        self.platform
            .prepare_render(&ui, unsafe { &mut (*self.window).window });

        self.draw_data = ui.render();

        if let None = self.ui_mesh {
            unsafe {
                self.ui_mesh = Some(GpuMeshMemory::from_draw_data(
                    _render_context,
                    &*self.draw_data,
                ));
            }
        } else {
            let mesh = self.ui_mesh.as_mut().unwrap();
            unsafe {
                mesh.update_from_draw_data(_render_context, &*self.draw_data);
            }
        }
    }

    fn end_frame(&mut self) {}
}
