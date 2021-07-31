use crate::{
    engine::{image::Texture, VulkanApplication},
    vulkan::{
        BufferLayout, DescriptorPool, DescriptorSet, LogicalDevice, UniformBuffer,
        UniformBufferObject, UniformBufferObjectTemplate,
    },
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{DescriptorSetLayout, Sampler, ShaderModule},
};
use std::{path::Path, ptr};

pub struct ShaderBuilder<'a> {
    textures: Option<&'a [Texture]>,
    sampler: Option<Sampler>,

    input_buffer_layout: Option<BufferLayout>,
    descriptor_set_layout: Option<DescriptorSetLayout>,
    descriptor_pool: Option<DescriptorPool>,
    write_descriptor_sets: Vec<vk::WriteDescriptorSet>,
    uniform_buffer: Option<UniformBuffer<UniformBufferObject>>,
    vertex_shader: &'static str,
    fragment_shader: &'static str,
    swapchain_images: usize,

    application: &'a VulkanApplication,
}

impl<'a> ShaderBuilder<'a> {
    pub fn builder(
        application: &'a VulkanApplication,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
        swapchain_images: usize,
    ) -> Self {
        ShaderBuilder {
            vertex_shader,
            fragment_shader,
            swapchain_images,
            application,

            textures: None,
            sampler: None,
            input_buffer_layout: None,
            descriptor_set_layout: None,
            descriptor_pool: None,
            write_descriptor_sets: vec![],
            uniform_buffer: None,
        }
    }

    pub fn with_input_buffer_layout(
        &mut self,
        input_buffer_layout: BufferLayout,
    ) -> &mut ShaderBuilder<'a> {
        self.input_buffer_layout = Some(input_buffer_layout);
        self
    }

    pub fn with_textures(
        &mut self,
        textures: &'a [Texture],
        sampler: Sampler,
    ) -> &mut ShaderBuilder<'a> {
        self.textures = Some(textures);
        self.sampler = Some(sampler);
        self
    }

    pub fn with_descriptor_pool(
        &mut self,
        descriptor_types: &[vk::DescriptorType],
    ) -> &mut ShaderBuilder<'a> {
        self.descriptor_pool = Some(DescriptorPool::new(
            &self.application.device,
            descriptor_types,
            self.swapchain_images,
        ));
        self
    }

    pub fn with_write_descriptor_sets(
        &mut self,
        write_descriptor_sets: Vec<vk::WriteDescriptorSet>,
    ) -> &mut ShaderBuilder<'a> {
        self.write_descriptor_sets = write_descriptor_sets;

        self
    }

    pub fn with_write_descriptor_layout(
        &mut self,
        layout_binding: &[vk::DescriptorSetLayoutBinding],
    ) -> &mut ShaderBuilder<'a> {
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(layout_binding)
            .build();

        self.descriptor_set_layout = Some(unsafe {
            self.application
                .device
                .create_descriptor_set_layout(&layout_create_info, None)
                .expect("failed to create descriptor set layout!")
        });

        self
    }

    pub fn build<U: UniformBufferObjectTemplate>(mut self) -> ShaderSet<U> {
        let uniform_buffer_object = U::default();

        if let None = self.descriptor_pool {
            self.with_descriptor_pool(&[]);
        }

        if let None = self.descriptor_set_layout {
            self.with_write_descriptor_layout(&[]);
        }

        let uniform_buffer = UniformBuffer::<U>::new(
            &self.application.instance,
            &self.application.device,
            self.swapchain_images,
        );

        let descriptor_pool = self.descriptor_pool.unwrap();
        let descriptor_sets = descriptor_pool.create_descriptor_sets::<U>(
            &self.application.device,
            self.descriptor_set_layout.unwrap(),
            self.write_descriptor_sets,
            &uniform_buffer,
        );

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

            uniform_buffer,
            uniform_buffer_object,

            descriptor_sets,
            descriptor_pool,
            descriptor_set_layout: self.descriptor_set_layout.unwrap(),
            input_buffer_layout: self.input_buffer_layout.unwrap(),
        }
    }
}

pub struct ShaderSet<U: UniformBufferObjectTemplate> {
    vertex_shader_module: vk::ShaderModule,
    uniform_buffer_object: U,
    fragment_shader_module: ShaderModule,

    pub descriptor_set_layout: DescriptorSetLayout,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub uniform_buffer: UniformBuffer<U>,
    pub descriptor_pool: DescriptorPool,
    pub input_buffer_layout: BufferLayout,
}

impl<U: UniformBufferObjectTemplate> ShaderSet<U> {
    pub fn fragment_shader(&self) -> vk::ShaderModule {
        self.fragment_shader_module
    }

    pub fn vertex_shader(&self) -> vk::ShaderModule {
        self.vertex_shader_module
    }

    pub fn uniform_mut(&mut self) -> &mut U {
        &mut self.uniform_buffer_object
    }

    pub fn update_uniform(&mut self, device: &LogicalDevice, current_image: usize) {
        let ubos = [self.uniform_buffer_object.clone()];

        let buffer_size = (std::mem::size_of::<U>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = device
                .map_memory(
                    self.uniform_buffer.buffers_memory(current_image),
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut U;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            device.unmap_memory(self.uniform_buffer.buffers_memory(current_image));
        }
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
