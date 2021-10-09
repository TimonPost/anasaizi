use crate::{
    engine::{BufferLayout, UniformObjectTemplate, VulkanApplication},
    vulkan::{LogicalDevice, ShaderIo},
};
use ash::{version::DeviceV1_0, vk, vk::ShaderModule};
use std::{path::Path, ptr};

pub struct ShaderBuilder<'a> {
    input_buffer_layout: Option<BufferLayout>,
    shader_io: Option<ShaderIo>,

    vertex_shader: &'static str,
    fragment_shader: &'static str,
    swapchain_images: usize,

    application: &'a VulkanApplication,
}

impl<'a> ShaderBuilder<'a> {
    /// Creates a new shader builder.
    pub fn builder(
        application: &'a VulkanApplication,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
        swapchain_images: usize,
    ) -> ShaderBuilder<'a> {
        ShaderBuilder::<'a> {
            vertex_shader,
            fragment_shader,
            swapchain_images,
            application,

            input_buffer_layout: None,
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

        ShaderSet {
            vertex_shader_module,
            fragment_shader_module,
            io: self.shader_io.unwrap(),
            shader_flag: ShaderFlags::empty()
        }
    }
}

/// A Vulkan Shader.
///
/// This shader contains the following data:
/// - Uniform buffer and object.
/// - Input buffer layout
/// - Descriptor pool, set, layout
pub struct ShaderSet {
    vertex_shader_module: vk::ShaderModule,
    fragment_shader_module: ShaderModule,
    pub io: ShaderIo,
    pub shader_flag: ShaderFlags
}

impl ShaderSet {

}

impl ShaderSet {
    pub fn get_descriptor_sets(&self, frame: usize, _texture: String) -> Vec<vk::DescriptorSet> {
        vec![*self.io.descriptor_sets[frame]]
    }

    pub fn descriptor_set_layout(&self) -> Vec<vk::DescriptorSetLayout> {
        vec![self.io.descriptor_set_layout]
    }
}

impl ShaderSet {
    pub fn fragment_shader(&self) -> vk::ShaderModule {
        self.fragment_shader_module
    }

    pub fn vertex_shader(&self) -> vk::ShaderModule {
        self.vertex_shader_module
    }

    pub fn add_uniform_object<U: UniformObjectTemplate + 'static>(&mut self, uniform_object: U) {
        self.io
            .uniform_buffer_objects
            .push(Box::new(uniform_object));
    }

    pub fn update_uniform<U: UniformObjectTemplate + Clone + 'static>(
        &mut self,
        device: &LogicalDevice,
        current_image: usize,
        object_index: usize,
        update_fn: &dyn Fn(&mut U),
    ) {
        if self.io.uniform_buffers.is_empty() {
            panic!("Trying to update shader uniform without uniform buffer.");
        }

        let uniform_object = if let Some(ubo) = self.io.uniform_buffer_objects.get_mut(object_index)
        {
            ubo.clone()
        } else {
            panic!(
                "Could not get uniformbuffer object with index: {}",
                object_index
            );
        };

        let uniform_buffer = if let Some(uniform_buffer) = self.io.uniform_buffers.get(object_index)
        {
            uniform_buffer
        } else {
            panic!("Could not get uniformbuffer with index: {}", object_index);
        };

        assert_eq!(uniform_buffer.uniform_object_size, uniform_object.size());

        let uniform_object_any = uniform_object.as_any();

        if let Some(obj) = uniform_object_any.downcast_ref::<U>() {
            let mut casted_uniform_object: U = (*obj).clone();

            update_fn(&mut casted_uniform_object);

            let updating_ubos = [casted_uniform_object];

            let buffer_size = (obj.size() * updating_ubos.len()) as u64;

            unsafe {
                let data_ptr = device
                    .map_memory(
                        uniform_buffer.buffers_memory(current_image),
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("Failed to Map Memory") as *mut U;

                data_ptr.copy_from_nonoverlapping(updating_ubos.as_ptr(), updating_ubos.len());

                device.unmap_memory(uniform_buffer.buffers_memory(current_image));
            };
        } else {
            println!("Could not cast the uniform object to its specific implementation.");
        }
    }

    fn add_defines(source: &str, defines: &[String]) -> String {
        // insert preprocessor defines after #version if exists
        // (#version must occur before any other statement in the program)
        let defines = defines.iter()
            .map(|define| format!("#define {}", define))
            .collect::<Vec<_>>()
            .join("\n");
        let mut lines: Vec<_> = source.lines().collect();
        if let Some(version_line) = lines.iter().position(|l| l.starts_with("#version")) {
            lines.insert(version_line+1, &defines);
        }
        else {
            lines.insert(0, &defines);
        }
        lines.join("\n")
    }

    /// Destroy the shader and its components:
    ///
    /// - Fragment shader module
    /// - Vertex shader module
    /// - Description set layout
    /// - Uniform buffer
    /// - Descriptor pool
    pub unsafe fn destroy(&self, device: &LogicalDevice) {
        device.destroy_shader_module(self.fragment_shader(), None);
        device.destroy_shader_module(self.vertex_shader(), None);

        self.io.destroy(device);
    }

    fn create_shader_module(device: &LogicalDevice, code: Vec<u8>) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
        };

        unsafe {
            device
                .create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create Shader Module!")
        }
    }

    fn read_shader_code(shader_path: &Path) -> Vec<u8> {
        use std::{fs::File, io::Read};

        let spv_file = File::open(shader_path)
            .expect(&format!("Failed to find spv file at {:?}", shader_path));
        let bytes_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();

        bytes_code
    }
}

use bitflags::bitflags;
use std::fmt::Display;
use crate::libs::imgui::__core::fmt::Formatter;
use crate::engine::gltf::GltfPBRShaderConstants;
use std::ffi::c_void;
use crate::libs::ash::vk::SpecializationInfo;

bitflags! {
    /// Flags matching the defines in the PBR shader
    pub struct ShaderFlags: u16 {
        // vertex shader + fragment shader
        const HAS_NORMALS           = 1;
        const HAS_TANGENTS          = 1 << 1;
        const HAS_UV                = 1 << 2;
        const HAS_COLORS            = 1 << 3;

        // fragment shader only
        const USE_IBL               = 1 << 4;
        const HAS_BASECOLORMAP      = 1 << 5;
        const HAS_NORMALMAP         = 1 << 6;
        const HAS_EMISSIVEMAP       = 1 << 7;
        const HAS_METALROUGHNESSMAP = 1 << 8;
        const HAS_OCCLUSIONMAP      = 1 << 9;
        const USE_TEX_LOD           = 1 << 10;
    }
}

impl ShaderFlags {
    pub fn as_strings(self) -> Vec<String> {
        (0..15)
            .map(|i| 1u16 << i)
            .filter(|i| self.bits & i != 0)
            .map(|i| format!("{:?}", ShaderFlags::from_bits_truncate(i)))
            .collect()
    }
}

impl From<ShaderFlags> for GltfPBRShaderConstants {
    fn from(flags: ShaderFlags) -> Self {
        let mut constants = GltfPBRShaderConstants::default();

        println!("flags: {:?}", flags);

        if flags.contains(ShaderFlags::HAS_BASECOLORMAP) { constants.has_basecolormap = 1; }
        if flags.contains(ShaderFlags::HAS_EMISSIVEMAP) {constants.has_emissivemap = 1;}
        if flags.contains(ShaderFlags::HAS_METALROUGHNESSMAP) {constants.has_metalroughnessmap = 1;}
        if flags.contains(ShaderFlags::HAS_NORMALMAP) {constants.has_normalmap = 1;}
        if flags.contains(ShaderFlags::HAS_OCCLUSIONMAP) {constants.has_occlusionmap = 1;}

        if flags.contains(ShaderFlags::HAS_TANGENTS) {constants.has_tangents = 1;}
        if flags.contains(ShaderFlags::HAS_NORMALS) {constants.has_normals = 1;}
        if flags.contains(ShaderFlags::HAS_UV) {constants.has_uvs = 1;}
        if flags.contains(ShaderFlags::HAS_COLORS) {constants.has_colors = 1;}

        if flags.contains(ShaderFlags::USE_IBL) {constants.use_ibl = 1;}

        constants
    }
}


impl Display for  ShaderFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "base: {} colors: {}, emisive: {}, roughness: {}, normal: {}, occlusion: {}, tangents: {}",
             self.contains(ShaderFlags::HAS_BASECOLORMAP),
             self.contains(ShaderFlags::HAS_COLORS),
             self.contains(ShaderFlags::HAS_EMISSIVEMAP),
             self.contains(ShaderFlags::HAS_METALROUGHNESSMAP),
             self.contains(ShaderFlags::HAS_NORMALMAP),
             self.contains(ShaderFlags::HAS_OCCLUSIONMAP),
             self.contains(ShaderFlags::HAS_TANGENTS)
        )
    }
}

