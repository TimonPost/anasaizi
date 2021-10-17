use crate::{
    engine::RenderContext,
    libs::imgui::DrawData,
    math::{Vector3, Vertex},
    vulkan::{GPUBuffer, VkLogicalDevice},
};

/// Mesh that holds the allocated vertex, index buffer and the model transformation.
#[derive(Clone)]
pub struct GpuMeshMemory {
    vertex_buffer: GPUBuffer,
    index_buffer: GPUBuffer,

    pub texture_id: i32,
}

impl GpuMeshMemory {
    /// Creates a new `Mesh` from the given allocated vertex and index buffer.
    pub fn new(
        vertex_buffer: GPUBuffer,
        index_buffer: GPUBuffer,
        texture_id: i32,
    ) -> GpuMeshMemory {
        GpuMeshMemory {
            vertex_buffer,
            index_buffer,
            texture_id,
        }
    }

    /// Creates a new `Mesh` from the given raw vertices and indices.
    pub fn from_raw<U: 'static>(
        render_context: &RenderContext,
        vertices: Vec<U>,
        indices: Vec<u32>,
        texture_id: i32,
    ) -> GpuMeshMemory {
        let vertex_buffer = GPUBuffer::create(render_context, &vertices);
        let index_buffer = GPUBuffer::create(render_context, &indices);

        GpuMeshMemory::new(vertex_buffer, index_buffer, texture_id)
    }

    /// Creates a new `Mesh` from the given imgui `DrawData`.
    pub fn from_draw_data(render_context: &RenderContext, draw_data: &DrawData) -> GpuMeshMemory {
        let (vertices, indices) = Self::get_vertices_and_indices(draw_data);

        let vertex_buffer = GPUBuffer::create::<Vertex>(render_context, &vertices);
        let index_buffer = GPUBuffer::create(render_context, &indices);

        GpuMeshMemory::new(vertex_buffer, index_buffer, -1)
    }

    /// Updates the mesh with the given imgui `DrawData`.
    ///
    /// This function will either reallocate, destroy, extend memory based up on the given `DrawData`.
    pub fn update_from_draw_data(&mut self, render_context: &RenderContext, draw_data: &DrawData) {
        let device = render_context.device();

        let (vertices, indices) = Self::get_vertices_and_indices(draw_data);

        // if vertex buffer is outdated
        if draw_data.total_vtx_count as usize > self.vertex_buffer.element_count() {
            self.vertex_buffer.destroy(device);

            let vertex_buffer = GPUBuffer::create::<Vertex>(render_context, &vertices);

            self.vertex_buffer = vertex_buffer;
        } else {
            // Update buffer content with new draw data.
            self.vertex_buffer.update_buffer_content(device, &vertices);
        }

        // if index buffer is outdated
        if draw_data.total_idx_count as usize > self.index_buffer.element_count() {
            self.index_buffer.destroy(device);

            let index_buffer = GPUBuffer::create(render_context, &indices);
            self.index_buffer = index_buffer;
        } else {
            // Update buffer content with new draw data.
            self.index_buffer.update_buffer_content(device, &indices);
        }
    }

    /// Returns the `Mesh` its vertex buffer.
    pub fn vertex_buffer(&self) -> &GPUBuffer {
        &self.vertex_buffer
    }

    /// Returns the `Mesh` its index buffer.
    pub fn index_buffer(&self) -> &GPUBuffer {
        &self.index_buffer
    }

    /// Returns the number of vertices.
    pub fn vertices_count(&self) -> usize {
        self.vertex_buffer.element_count()
    }

    /// Returns the number of indices.
    pub fn indices_count(&self) -> usize {
        self.index_buffer.element_count()
    }

    /// Destroys `Mesh` contents:
    /// - IndexBuffer and its memory.
    /// - VertexBuffer and its memory.
    pub fn destroy(&self, device: &VkLogicalDevice) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
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
                        normal: Vector3::default(),
                        tangent: Vector3::default(),
                        bitangent: Vector3::default(),
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
