use std::path::Path;
use ash::vk;
use std::ptr;
use ash::version::DeviceV1_0;
use crate::LogicalDevice;
use std::collections::HashMap;
use std::ops::Deref;

pub struct Shaders {
    shaders: HashMap<&'static str, Shader>
}

impl Shaders {
    pub fn new() -> Shaders {
        Shaders { shaders: HashMap::new() }
    }

    pub fn add_shader(&mut self, label: &'static str, shader: Shader) {
        self.shaders.insert(label, shader);
    }

    pub fn shader(&self, label: &'static str) -> &Shader {
        self.shaders.get(label).expect(format!("Could not find shader with label: {}", label).as_str())
    }
}

pub struct Shader {
    shader_path: & 'static str,
    shader_module: vk::ShaderModule
}

impl Shader {
    pub fn new(device: &LogicalDevice, shader_path: & 'static str) -> Shader {
        let shader_code = Self::read_shader_code(Path::new(shader_path));

        let shader_module = Self::create_shader_module(device, shader_code);

        Shader {
            shader_path,
            shader_module
        }
    }

    pub fn create_shader_module(device: &LogicalDevice, code: Vec<u8>) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
        };

        unsafe {
            device
                .logical_device()
                .create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create Shader Module!")
        }
    }

    pub fn read_shader_code(shader_path: &Path) -> Vec<u8> {
        use std::fs::File;
        use std::io::Read;

        let spv_file =
            File::open(shader_path).expect(&format!("Failed to find spv file at {:?}", shader_path));
        let bytes_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();

        bytes_code
    }

}

impl Deref for Shader {
    type Target = vk::ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.shader_module
    }
}