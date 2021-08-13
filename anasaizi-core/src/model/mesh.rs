use crate::{
    engine::{RenderContext, VulkanApplication},
    math::Vertex,
    reexports::imgui::DrawData,
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
use nalgebra::{Matrix4, Vector3};

/// Mesh that holds the allocated vertex, index buffer and the model transformation.
pub struct Mesh {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,

    pub scale_transform: nalgebra::Matrix4<f32>,
    pub rotate_transform: nalgebra::Matrix4<f32>,
    pub translate_transform: nalgebra::Matrix4<f32>,

    pub texture_id: i32,
}

impl Mesh {
    /// Creates a new `Mesh` from the given allocated vertex and index buffer.
    pub fn new(vertex_buffer: VertexBuffer, index_buffer: IndexBuffer, texture_id: i32) -> Mesh {
        let mut identity = Matrix4::default();
        identity.fill_with_identity();

        Mesh {
            vertex_buffer,
            scale_transform: identity,
            translate_transform: identity,
            rotate_transform: identity,
            index_buffer,
            texture_id,
        }
    }

    pub fn from_raw(
        render_context: &RenderContext,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        texture_id: i32,
    ) -> Mesh {
        let vertex_buffer = VertexBuffer::create(render_context, &vertices);
        let index_buffer = IndexBuffer::create(render_context, &indices);

        Mesh::new(vertex_buffer, index_buffer, texture_id)
    }

    /// Creates a new `Mesh` from the given imgui `DrawData`.
    pub fn from_draw_data(render_context: &RenderContext, draw_data: &DrawData) -> Mesh {
        let (vertices, indices) = Self::get_vertices_and_indices(draw_data);

        let vertex_buffer = VertexBuffer::create::<Vertex>(render_context, &vertices);
        let index_buffer = IndexBuffer::create(render_context, &indices);

        Mesh::new(vertex_buffer, index_buffer, -1)
    }

    pub fn rotate(&mut self, rotate: Vector3<f32>) {
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

    pub fn model_transform(&self) -> nalgebra::Matrix4<f32> {
        return self.rotate_transform * self.scale_transform * self.translate_transform;
    }

    /// Destroys `Mesh` contents:
    /// - IndexBuffer and its memory.
    /// - VertexBuffer and its memory.
    pub fn destroy(&self, device: &LogicalDevice) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
    }

    pub fn push_constants<U: UniformBufferObjectTemplate>(
        &self,
        device: &ash::Device,
        command_buffer: &CommandBuffer,
        pipeline: &vulkan::Pipeline<U>,
    ) {
        // Push the model matrix using push constants.
        let transform = MeshPushConstants {
            model_matrix: self.model_transform(),
            texture_id: self.texture_id,
        };

        unsafe {
            let push_constants = any_as_u8_slice(&transform);

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
