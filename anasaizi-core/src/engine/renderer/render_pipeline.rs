use crate::{
    engine::GpuMeshMemory,
    reexports::imgui::DrawData,
    utils::any_as_u8_slice,
    vulkan::{
        structures::ObjectIdPushConstants, CommandBuffers, CommandPool, IndexBuffer, Instance,
        LogicalDevice, MeshPushConstants, Pipeline, Queue, QueueFamilyIndices, UniformBufferObject,
        UniformObjectTemplate, VertexBuffer,
    },
};
use ash::{version::DeviceV1_0, vk, vk::CommandBuffer, Device};
use nalgebra::Matrix4;
use std::ptr;

pub struct RenderPipeline {
    active_command_buffer: *const CommandBuffer,
    device: *const LogicalDevice,
    pub active_mesh: *const GpuMeshMemory,
    active_pipeline: *const Pipeline,

    pub index_count: u32,
    pub index_offset: u32,
    pub vertex_offset: u32,

    active_image_index: usize,
}

impl RenderPipeline {
    pub fn new(
        device: &LogicalDevice,
        command_buffer: &CommandBuffer,
        active_image: usize,
    ) -> RenderPipeline{
        RenderPipeline {
            active_command_buffer: command_buffer,
            device,

            active_mesh: ptr::null(),
            active_pipeline: ptr::null(),

            index_count: 0,
            index_offset: 0,
            vertex_offset: 0,
            active_image_index: active_image,
        }
    }

    pub fn bind_pipeline(&mut self, pipeline: &Pipeline, command_buffer: &CommandBuffers) {
        command_buffer.bind_pipeline(self.device(), pipeline);
        self.active_pipeline = pipeline;
    }

    pub fn set_view_port(&self, x: f32, y: f32, width: f32, height: f32) {
        let viewports = [vk::Viewport {
            width,
            height,
            x,
            y,
            max_depth: 1.0,
            ..Default::default()
        }];

        unsafe { (*self.device).cmd_set_viewport(*self.active_command_buffer, 0, &viewports) };
    }

    pub fn set_scissors(&self, clip_x: f32, clip_y: f32, clip_w: f32, clip_h: f32) {
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D {
                x: clip_x as _,
                y: clip_y as _,
            },
            extent: vk::Extent2D {
                width: clip_w as _,
                height: clip_h as _,
            },
        }];
        unsafe {
            self.device()
                .cmd_set_scissor(*self.active_command_buffer, 0, &scissors);
        }
    }

    pub fn set_mesh(&mut self, mesh: &GpuMeshMemory) {
        self.active_mesh = mesh;
        self.index_count = mesh.indices_count() as u32;
    }

    pub fn render_mesh(&self) {
        self.bind_buffers();
        self.bind_descriptor_sets();
        self.draw_indexed();
    }

    pub fn push_mesh_constants<T: Sized>(&self, data: T) {
        // Push the model matrix using push constants.
        unsafe {
            self.active_mesh().push_constants::<T>(
                self.device(),
                self.active_command_buffer(),
                self.active_pipeline(),
                data,
            );
        };
    }

    pub fn push_ui_constants(&self, draw_data: &DrawData) {
        let orthographic = nalgebra::Orthographic3::new(
            0.0,
            draw_data.display_size[0],
            0.0,
            -draw_data.display_size[1],
            -1.0,
            1.0,
        );

        let mut matrix = orthographic.to_homogeneous();
        matrix[(1, 1)] = matrix[(1, 1)] * -1.0;

        unsafe {
            let push = any_as_u8_slice(&matrix);
            self.device().cmd_push_constants(
                *self.active_command_buffer,
                self.active_pipeline().layout(),
                vk::ShaderStageFlags::VERTEX,
                0,
                &push,
            )
        };
    }

    pub fn bind_buffers(&self) {
        unsafe {
            self.device().cmd_bind_index_buffer(
                *self.active_command_buffer,
                **self.active_mesh().index_buffer(),
                0,
                vk::IndexType::UINT32,
            );

            self.device().cmd_bind_vertex_buffers(
                *self.active_command_buffer,
                0,
                &[**self.active_mesh().vertex_buffer()],
                &[0],
            )
        };
    }

    fn active_pipeline(&self) -> &Pipeline {
        unsafe { &*self.active_pipeline }
    }

    fn active_command_buffer(&self) -> &CommandBuffer {
        unsafe { &*self.active_command_buffer }
    }

    fn active_mesh(&self) -> &GpuMeshMemory {
        unsafe { &*self.active_mesh }
    }

    fn device(&self) -> &LogicalDevice {
        unsafe { &*self.device }
    }

    fn bind_descriptor_sets(&self) {
        let sets = self
            .active_pipeline()
            .shader
            .get_descriptor_sets(self.active_image_index, String::from(""));

        unsafe {
            self.device().cmd_bind_descriptor_sets(
                *self.active_command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.active_pipeline().layout(),
                0,
                &sets,
                &[],
            );
        }
    }

    fn draw_indexed(&self) {
        unsafe {
            self.device().cmd_draw_indexed(
                *self.active_command_buffer,
                self.index_count as _,
                1,
                self.index_offset as u32,
                self.vertex_offset as i32,
                0,
            )
        }
    }
}

pub struct RenderContext {
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    instance: ash::Instance,
    device: LogicalDevice,
}

impl RenderContext {
    pub fn new(
        instance: &Instance,
        command_pool: &CommandPool,
        device: &LogicalDevice,
        graphics_queue: &Queue,
    ) -> RenderContext {
        RenderContext {
            graphics_queue: **graphics_queue,
            command_pool: **command_pool,
            instance: (**(instance)).clone(),
            device: device.clone(),
        }
    }
    pub fn queue_family_indices(&self) -> &QueueFamilyIndices {
        &self.device.queue_family_indices()
    }
    pub fn find_memory_type(
        &self,
        type_filter: u32,
        required_properties: vk::MemoryPropertyFlags,
    ) -> u32 {
        self.device
            .find_memory_type(type_filter, required_properties)
    }

    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    pub fn logical_device(&self) -> &LogicalDevice {
        &self.device
    }

    pub fn physical_device(&self) -> &vk::PhysicalDevice {
        self.device.physical_device()
    }

    pub fn raw_instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }
}
