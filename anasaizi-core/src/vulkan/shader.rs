use crate::{
    engine::image::Texture,
    vulkan::{
        DescriptorPool, DescriptorSet, Instance, LogicalDevice, UniformBuffer, UniformBufferObject,
        UniformBufferObjectTemplate,
    },
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{DescriptorSetLayout, ShaderModule},
};
use std::{collections::HashMap, ops::Deref, path::Path, ptr};

pub struct ShaderSet<U: UniformBufferObjectTemplate> {
    vertex_shader_module: vk::ShaderModule,
    uniform_buffer_object: U,
    fragment_shader_module: ShaderModule,

    pub descriptor_set_layout: DescriptorSetLayout,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub uniform_buffer: UniformBuffer<U>,
    pub descriptor_pool: DescriptorPool,
}

impl<U: UniformBufferObjectTemplate> ShaderSet<U> {
    pub fn new(
        instance: &Instance,
        device: &LogicalDevice,
        vertex_shader_path: &'static str,
        fragment_shader_path: &'static str,
        descriptor_set_layout: DescriptorSetLayout,
        swap_chain_image_count: usize,
        texture_sampler: vk::Sampler,
        texture: &Texture,
    ) -> ShaderSet<U> {
        let uniform_buffer_object = U::default();

        let descriptor_pool = DescriptorPool::new(&device, swap_chain_image_count);

        let uniform_buffer = UniformBuffer::<U>::new(&instance, &device, swap_chain_image_count);
        let descriptor_sets = descriptor_pool.create_descriptor_sets::<U>(
            &device,
            &uniform_buffer,
            descriptor_set_layout,
            texture_sampler,
            texture.image_view.clone(),
        );

        let vertex_shader_code = Self::read_shader_code(Path::new(vertex_shader_path));
        let vertex_shader_module = Self::create_shader_module(device, vertex_shader_code);

        let fragment_shader_code = Self::read_shader_code(Path::new(fragment_shader_path));
        let fragment_shader_module = Self::create_shader_module(device, fragment_shader_code);

        ShaderSet {
            vertex_shader_module,

            fragment_shader_module,

            uniform_buffer_object,
            descriptor_set_layout,
            descriptor_sets,
            uniform_buffer,
            descriptor_pool,
        }
    }

    pub fn fragment_shader(&self) -> vk::ShaderModule {
        self.fragment_shader_module
    }

    pub fn vertex_shader(&self) -> vk::ShaderModule {
        self.vertex_shader_module
    }

    pub fn update_uniform(&mut self, uniform: U) {
        self.uniform_buffer_object = uniform;
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
