use crate::{
    engine::{BufferLayout, VulkanApplication},
    vulkan::{
        DescriptorPool, DescriptorSet, LogicalDevice, ShaderIo, UniformBuffer,
        UniformBufferObjectTemplate,
    },
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{DescriptorSetLayout, ShaderModule},
};
use std::{path::Path, ptr};

pub struct ShaderBuilder<'a, U: UniformBufferObjectTemplate> {
    input_buffer_layout: Option<BufferLayout>,
    descriptors: Option<ShaderIo<U>>,

    vertex_shader: &'static str,
    fragment_shader: &'static str,
    swapchain_images: usize,

    application: &'a VulkanApplication,
}

impl<'a, U: UniformBufferObjectTemplate> ShaderBuilder<'a, U> {
    /// Creates a new shader builder.
    pub fn builder(
        application: &'a VulkanApplication,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
        swapchain_images: usize,
    ) -> ShaderBuilder<'a, U> {
        ShaderBuilder::<'a, U> {
            vertex_shader,
            fragment_shader,
            swapchain_images,
            application,

            input_buffer_layout: None,
            descriptors: None,
        }
    }

    pub fn with_descriptors(&mut self, descriptors: ShaderIo<U>) -> &mut ShaderBuilder<'a, U> {
        self.descriptors = Some(descriptors);
        self
    }

    /// Build shader.
    pub fn build(mut self) -> ShaderSet<U> {
        let vertex_shader_code = ShaderSet::<U>::read_shader_code(Path::new(self.vertex_shader));
        let vertex_shader_module =
            ShaderSet::<U>::create_shader_module(&self.application.device, vertex_shader_code);

        let fragment_shader_code =
            ShaderSet::<U>::read_shader_code(Path::new(self.fragment_shader));
        let fragment_shader_module =
            ShaderSet::<U>::create_shader_module(&self.application.device, fragment_shader_code);

        ShaderSet {
            vertex_shader_module,
            fragment_shader_module,

            io: self.descriptors.unwrap(),
        }
    }
}

/// A Vulkan Shader.
///
/// This shader contains the following data:
/// - Uniform buffer and object.
/// - Input buffer layout
/// - Descriptor pool, set, layout
pub struct ShaderSet<U: UniformBufferObjectTemplate> {
    vertex_shader_module: vk::ShaderModule,
    fragment_shader_module: ShaderModule,

    pub io: ShaderIo<U>,
}

impl<U: UniformBufferObjectTemplate> ShaderSet<U> {
    pub fn get_descriptor_sets(&self, frame: usize, texture: String) -> Vec<vk::DescriptorSet> {
        let mut result = vec![*self.io.descriptor_sets[frame]];

        // if let Some(set) = self.io.texture_descriptor_sets.texture(texture, frame) {
        //     result.push(*set);
        // }

        result
    }

    pub fn descriptor_set_layout(&self) -> Vec<vk::DescriptorSetLayout> {
        vec![self.io.descriptor_set_layout]
    }
}

impl<U: UniformBufferObjectTemplate> ShaderSet<U> {
    pub fn fragment_shader(&self) -> vk::ShaderModule {
        self.fragment_shader_module
    }

    pub fn vertex_shader(&self) -> vk::ShaderModule {
        self.vertex_shader_module
    }

    pub fn uniform(&self) -> &U {
        &self.io.uniform_buffer_object
    }

    pub fn uniform_mut(&mut self) -> &mut U {
        &mut self.io.uniform_buffer_object
    }

    /// Write the uniform buffer object to shader memory.
    pub fn update_uniform(&self, device: &LogicalDevice, current_image: usize) {
        let ubos = [self.io.uniform_buffer_object.clone()];

        let buffer_size = (self.io.uniform_buffer_object.size() * ubos.len()) as u64;

        unsafe {
            let data_ptr = device
                .map_memory(
                    self.io.uniform_buffer.buffers_memory(current_image),
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut U;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            device.unmap_memory(self.io.uniform_buffer.buffers_memory(current_image));
        }
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
