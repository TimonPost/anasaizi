use crate::{
    engine::VulkanApplication,
    vulkan::{
        shader::{shader_flags::ShaderFlags, shader_set::ShaderSet},
        ShaderIo,
    },
};
use std::path::Path;

pub struct ShaderBuilder<'a> {
    shader_io: Option<ShaderIo>,

    vertex_shader: &'static str,
    fragment_shader: &'static str,

    application: &'a VulkanApplication,
}

impl<'a> ShaderBuilder<'a> {
    /// Creates a new shader builder.
    pub fn builder(
        application: &'a VulkanApplication,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
    ) -> ShaderBuilder<'a> {
        ShaderBuilder::<'a> {
            vertex_shader,
            fragment_shader,
            application,
            shader_io: None,
        }
    }

    pub fn with_descriptors(mut self, descriptors: ShaderIo) -> ShaderBuilder<'a> {
        self.shader_io = Some(descriptors);
        self
    }

    /// Build shader.
    pub fn build(self) -> ShaderSet {
        let vertex_shader_code = ShaderSet::read_shader_code(Path::new(self.vertex_shader));
        let vertex_shader_module =
            ShaderSet::create_shader_module(&self.application.device, vertex_shader_code);

        let fragment_shader_code = ShaderSet::read_shader_code(Path::new(self.fragment_shader));
        let fragment_shader_module =
            ShaderSet::create_shader_module(&self.application.device, fragment_shader_code);

        ShaderSet::new(
            vertex_shader_module,
            fragment_shader_module,
            self.shader_io.unwrap(),
            ShaderFlags::empty(),
        )
    }
}
