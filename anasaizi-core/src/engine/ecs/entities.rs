use crate::{
    engine::{RenderContext, VulkanApplication},
    math::Vertex,
    reexports::{
        imgui::{DrawData, __core::ops::RangeInclusive},
        nalgebra::{Matrix4, Vector3},
    },
    utils::any_as_u8_slice,
    vulkan,
    vulkan::{
        CommandPool, IndexBuffer, Instance, LogicalDevice, MeshPushConstants, Queue,
        UniformBufferObjectTemplate, VertexBuffer,
    },
};
use ash::{
    version::DeviceV1_0,
    vk,
    vk::{CommandBuffer, Pipeline},
};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub scale_transform: nalgebra::Matrix4<f32>,
    pub rotate_transform: nalgebra::Matrix4<f32>,
    pub translate_transform: nalgebra::Matrix4<f32>,

    pub unit_scale: f32,
    rotation_factor: Vector3<f32>,
}

impl Transform {
    pub fn new(unit_scale: f32) -> Transform {
        let mut identity = Matrix4::default();
        identity.fill_with_identity();

        Transform {
            scale_transform: identity,
            rotate_transform: identity,
            translate_transform: identity,
            rotation_factor: Vector3::default(),
            unit_scale,
        }
    }

    pub fn unit_scale(&self) -> RangeInclusive<f32> {
        0.0..=self.unit_scale
    }

    pub fn with_scale(mut self, factor: f32) -> Transform {
        self.scale(factor);
        self
    }

    pub fn with_translate(mut self, translate: Vector3<f32>) -> Transform {
        self.translate(translate);
        self
    }

    pub fn with_rotation(mut self, rotation: Vector3<f32>) -> Transform {
        self.rotate(rotation);
        self
    }

    pub fn rotate(&mut self, rotate: Vector3<f32>) {
        self.rotation_factor = rotate;
        self.rotate_transform = Matrix4::new_rotation(rotate);
    }
    pub fn translate(&mut self, translate: Vector3<f32>) {
        let translate_matrix = Matrix4::new(
            1.0,
            0.0,
            0.0,
            translate[0],
            0.0,
            1.0,
            0.0,
            translate[1],
            0.0,
            0.0,
            1.0,
            translate[2],
            0.0,
            0.0,
            0.0,
            1.0,
        );

        self.translate_transform = translate_matrix;
    }

    pub fn scale(&mut self, factor: f32) {
        let scale_matrix = Matrix4::new(
            factor, 0.0, 0.0, 0.0, 0.0, factor, 0.0, 0.0, 0.0, 0.0, factor, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        self.scale_transform = scale_matrix;
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_transform[(0, 0)]
    }

    pub fn rotation_factor(&self) -> Vector3<f32> {
        self.rotation_factor
    }

    pub fn translate_factor(&self) -> Vector3<f32> {
        let x = self.translate_transform[(3, 0)];
        let y = self.translate_transform[(3, 0)];
        let z = self.translate_transform[(3, 0)];

        Vector3::new(x, y, z)
    }

    pub fn model_transform(&self) -> nalgebra::Matrix4<f32> {
        return self.rotate_transform * self.scale_transform * self.translate_transform;
    }
}

/// Mesh that holds the allocated vertex, index buffer and the model transformation.
pub struct GpuMeshMemory {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,

    pub texture_id: i32,
}

impl GpuMeshMemory {
    /// Creates a new `Mesh` from the given allocated vertex and index buffer.
    pub fn new(
        vertex_buffer: VertexBuffer,
        index_buffer: IndexBuffer,
        texture_id: i32,
    ) -> GpuMeshMemory {
        GpuMeshMemory {
            vertex_buffer,
            index_buffer,
            texture_id,
        }
    }

    pub fn from_raw(
        render_context: &RenderContext,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        texture_id: i32,
    ) -> GpuMeshMemory {
        let vertex_buffer = VertexBuffer::create(render_context, &vertices);
        let index_buffer = IndexBuffer::create(render_context, &indices);

        GpuMeshMemory::new(vertex_buffer, index_buffer, texture_id)
    }

    /// Creates a new `Mesh` from the given imgui `DrawData`.
    pub fn from_draw_data(render_context: &RenderContext, draw_data: &DrawData) -> GpuMeshMemory {
        let (vertices, indices) = Self::get_vertices_and_indices(draw_data);

        let vertex_buffer = VertexBuffer::create::<Vertex>(render_context, &vertices);
        let index_buffer = IndexBuffer::create(render_context, &indices);

        GpuMeshMemory::new(vertex_buffer, index_buffer, -1)
    }

    /// Updates the mesh with the given imgui `DrawData`.
    ///
    /// This function will either reallocate, destroy, extend memory based up on the given `DrawData`.
    pub fn update_from_draw_data(&mut self, render_context: &RenderContext, draw_data: &DrawData) {
        let device = render_context.device();

        let (vertices, indices) = Self::get_vertices_and_indices(draw_data);

        // if vertex buffer is outdated
        if draw_data.total_vtx_count as usize > self.vertex_buffer.vertices_count() {
            self.vertex_buffer.destroy(device);

            let vertex_buffer = VertexBuffer::create::<Vertex>(render_context, &vertices);

            self.vertex_buffer = vertex_buffer;
        } else {
            // Update buffer content with new draw data.
            self.vertex_buffer.update_buffer_content(device, &vertices);
        }

        // if index buffer is outdated
        if draw_data.total_idx_count as usize > self.index_buffer.indices_count() {
            self.index_buffer.destroy(device);

            let index_buffer = IndexBuffer::create(render_context, &indices);
            self.index_buffer = index_buffer;
        } else {
            // Update buffer content with new draw data.
            self.index_buffer.update_buffer_content(device, &indices);
        }
    }

    /// Returns the `Mesh` its `VertexBuffer`.
    pub fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    /// Returns the `Mesh` its `IndexBuffer`.
    pub fn index_buffer(&self) -> &IndexBuffer {
        &self.index_buffer
    }

    /// Returns the number of vertices.
    pub fn vertices_count(&self) -> usize {
        self.vertex_buffer.vertices_count()
    }

    /// Returns the number of indices.
    pub fn indices_count(&self) -> usize {
        self.index_buffer.indices_count()
    }

    /// Destroys `Mesh` contents:
    /// - IndexBuffer and its memory.
    /// - VertexBuffer and its memory.
    pub fn destroy(&self, device: &LogicalDevice) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
    }

    pub fn push_constants<U: UniformBufferObjectTemplate, T>(
        &self,
        device: &ash::Device,
        command_buffer: &CommandBuffer,
        pipeline: &vulkan::Pipeline<U>,
        data: T,
    ) {
        unsafe {
            let push_constants = any_as_u8_slice(&data);

            device.cmd_push_constants(
                *command_buffer,
                pipeline.layout(),
                vk::ShaderStageFlags::VERTEX,
                0,
                &push_constants,
            );
        }
    }

    /// Gets vertices and indices from the imgui `DrawData`.
    fn get_vertices_and_indices(draw_data: &DrawData) -> (Vec<Vertex>, Vec<u32>) {
        // load vertexes
        let vertex_count = draw_data.total_vtx_count as usize;
        let mut vertices: Vec<Vertex> = Vec::with_capacity(vertex_count);

        for draw_list in draw_data.draw_lists() {
            let vertexes = draw_list
                .vtx_buffer()
                .iter()
                .map(|vertex| {
                    Vertex {
                        pos: nalgebra::Vector3::new(vertex.pos[0], vertex.pos[1], 0.0),
                        // Glsl used color range `0 <= x <=1` and imgui `0 <= x <= 255`, therefore divide by 255.
                        color: nalgebra::Vector4::new(
                            vertex.col[0] as f32 / 255.0,
                            vertex.col[1] as f32 / 255.0,
                            vertex.col[2] as f32 / 255.0,
                            vertex.col[3] as f32 / 255.0,
                        ),
                        tex_coord: nalgebra::Vector2::new(vertex.uv[0], vertex.uv[1]),
                    }
                })
                .collect::<Vec<Vertex>>();

            vertices.extend_from_slice(&vertexes);
        }

        // load indices
        let index_count = draw_data.total_idx_count as usize;
        let mut indices = Vec::with_capacity(index_count);
        for draw_list in draw_data.draw_lists() {
            indices.extend_from_slice(
                &draw_list
                    .idx_buffer()
                    .iter()
                    .map(|index| *index as u32)
                    .collect::<Vec<u32>>(),
            );
        }

        (vertices, indices)
    }
}
