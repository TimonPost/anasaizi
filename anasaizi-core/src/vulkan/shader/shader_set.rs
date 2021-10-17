use crate::{
    engine::UniformObjectTemplate,
    vulkan::{shader::shader_flags::ShaderFlags, ShaderIo, VkLogicalDevice},
};
use ash::{version::DeviceV1_0, vk, vk::ShaderModule};
use std::{path::Path, ptr};

/// A Vulkan Shader.
///
/// This shader contains the following data:
/// - Uniform buffer and object.
/// - Input buffer layout
/// - Descriptor pool, set, layout
pub struct ShaderSet {
    vertex_module: vk::ShaderModule,
    fragment_module: ShaderModule,
    pub io: ShaderIo,
    pub flags: ShaderFlags,
}

impl ShaderSet {
    pub fn new(
        vertex_module: ShaderModule,
        fragment_module: ShaderModule,
        io: ShaderIo,
        flags: ShaderFlags,
    ) -> ShaderSet {
        ShaderSet {
            vertex_module,
            fragment_module,
            io,
            flags,
        }
    }

    pub fn get_descriptor_sets(&self, frame: usize, _texture: String) -> Vec<vk::DescriptorSet> {
        vec![*self.io.descriptor_sets[frame]]
    }

    pub fn descriptor_set_layout(&self) -> Vec<vk::DescriptorSetLayout> {
        vec![self.io.descriptor_set_layout]
    }
}

impl ShaderSet {
    pub fn fragment_shader(&self) -> vk::ShaderModule {
        self.fragment_module
    }

    pub fn vertex_shader(&self) -> vk::ShaderModule {
        self.vertex_module
    }

    pub fn add_uniform_object<U: UniformObjectTemplate + 'static>(&mut self, uniform_object: U) {
        self.io
            .uniform_buffer_objects
            .push(Box::new(uniform_object));
    }

    pub fn update_uniform<U: UniformObjectTemplate + Clone + 'static>(
        &mut self,
        device: &VkLogicalDevice,
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

    /// Destroy the shader and its components:
    ///
    /// - Fragment shader module
    /// - Vertex shader module
    /// - Description set layout
    /// - Uniform buffer
    /// - Descriptor pool
    pub unsafe fn destroy(&self, device: &VkLogicalDevice) {
        device.destroy_shader_module(self.fragment_shader(), None);
        device.destroy_shader_module(self.vertex_shader(), None);

        self.io.destroy(device);
    }

    pub(crate) fn create_shader_module(
        device: &VkLogicalDevice,
        code: Vec<u8>,
    ) -> vk::ShaderModule {
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

    pub(crate) fn read_shader_code(shader_path: &Path) -> Vec<u8> {
        use std::{fs::File, io::Read};

        let spv_file = File::open(shader_path)
            .expect(&format!("Failed to find spv file at {:?}", shader_path));
        let bytes_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();

        bytes_code
    }
}
