
use anasaizi_core::{
    engine::{image::Texture, Event, Layer, RenderContext, RenderLayer, VulkanApplication},
    model::Mesh,
    reexports::{
        imgui::{
            Context, DrawData, FontConfig, FontGlyphRanges, FontSource,
            TextureId,
        },
        imgui_winit_support::{HiDpiMode, WinitPlatform},
    },
    vulkan::{UniformBufferObject, Window},
};
use std::{mem, time::Duration};
use anasaizi_core::reexports::imgui::ImStr;


pub struct WindowData {
    pub object_translate: [f32; 3],
    pub object_rotate: [f32; 3],
    pub object_scale: [f32; 3],
}

pub struct ImguiLayer {
    pub platform: WinitPlatform,
    pub imgui_context: Context,
    pub ui_font_texture: Texture,
    pub data: WindowData,

    pub ui_mesh: Option<Mesh>,

    pub window: *mut Window,
    pub draw_data: *const DrawData,
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
            data: WindowData {
                object_scale: [0.0; 3],
                object_translate: [0.0; 3],
                object_rotate: [0.0; 3],
            },
            ui_mesh: None,
            window: &mut application.window,
            draw_data: std::ptr::null(),
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
            Event::Raw(event) => self.platform.handle_event(
                self.imgui_context.io_mut(),
                unsafe { &mut (*self.window).window },
                event,
            ),
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
            ui.input_float3(ImStr::from_utf8_with_nul_unchecked("Translate".as_bytes()), &mut self.data.object_translate)
                .build();
            ui.input_float3(ImStr::from_utf8_with_nul_unchecked("Rotate".as_bytes()), &mut self.data.object_rotate)
                .build();
            ui.input_float3(ImStr::from_utf8_with_nul_unchecked("Scale".as_bytes()), &mut self.data.object_scale)
                .build();
        }

        ui.separator();
        let mouse_pos = ui.io().mouse_pos;
        ui.text(format!(
            "Mouse Position: ({:.1},{:.1})",
            mouse_pos[0], mouse_pos[1]
        ));

        self.platform
            .prepare_render(&ui, unsafe { &mut (*self.window).window });

        self.draw_data = ui.render();
        if let None = self.ui_mesh {
            unsafe {
                self.ui_mesh = Some(Mesh::from_draw_data(_render_context, &*self.draw_data));
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
