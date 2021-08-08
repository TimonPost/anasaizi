use crate::{
    math::Vertex,
    reexports::imgui::DrawData,
    vulkan::{CommandPool, IndexBuffer, Instance, LogicalDevice, Queue, VertexBuffer},
};
use nalgebra::Matrix4;

/// Mesh that holds the allocated vertex, index buffer and the model transformation.
pub struct Mesh {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    model_transform: nalgebra::Matrix4<f32>,
}

impl Mesh {
    /// Creates a new `Mesh` from the given allocated vertex and index buffer.
    pub fn new(vertex_buffer: VertexBuffer, index_buffer: IndexBuffer) -> Mesh {
        Mesh {
            vertex_buffer,
            model_transform: Matrix4::default(),
            index_buffer,
        }
    }

    /// Creates a new `Mesh` from the given imgui `DrawData`.
    pub fn from_draw_data(
        instance: &Instance,
        device: &LogicalDevice,
        submit_queue: &Queue,
        command_pool: &CommandPool,
        draw_data: &DrawData,
    ) -> Mesh {
        let (vertices, indices) = Self::get_vertices_and_indices(draw_data);

        let vertex_buffer =
            VertexBuffer::create::<Vertex>(instance, device, &vertices, submit_queue, command_pool);
        let index_buffer =
            IndexBuffer::create(instance, device, &indices, submit_queue, command_pool);

        Mesh {
            vertex_buffer,
            index_buffer,
            model_transform: Matrix4::default(),
        }
    }

    /// Updates the mesh with the given imgui `DrawData`.
    ///
    /// This function will either reallocate, destroy, extend memory based up on the given `DrawData`.
    pub fn update_from_draw_data(
        &mut self,
        instance: &Instance,
        device: &LogicalDevice,
        submit_queue: &Queue,
        command_pool: &CommandPool,
        draw_data: &DrawData,
    ) {
        let (vertices, indices) = Self::get_vertices_and_indices(draw_data);

        // if vertex buffer is outdated
        if draw_data.total_vtx_count as usize > self.vertex_buffer.vertices_count() {
            self.vertex_buffer.destroy(device);

            let vertex_buffer = VertexBuffer::create::<Vertex>(
                instance,
                device,
                &vertices,
                submit_queue,
                command_pool,
            );

            self.vertex_buffer = vertex_buffer;
        } else {
            // Update buffer content with new draw data.
            self.vertex_buffer.update_buffer_content(&device, &vertices);
        }

        // if index buffer is outdated
        if draw_data.total_idx_count as usize > self.index_buffer.indices_count() {
            self.index_buffer.destroy(device);

            let index_buffer =
                IndexBuffer::create(instance, device, &indices, submit_queue, command_pool);
            self.index_buffer = index_buffer;
        } else {
            // Update buffer content with new draw data.
            self.index_buffer.update_buffer_content(&device, &indices);
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

    pub fn update_model_transform(&mut self, matrix: nalgebra::Matrix4<f32>) {
        self.model_transform = matrix;
    }

    pub fn model_transform(&self) -> &nalgebra::Matrix4<f32> {
        return &self.model_transform;
    }

    /// Destroys `Mesh` contents:
    /// - IndexBuffer and its memory.
    /// - VertexBuffer and its memory.
    pub fn destroy(&self, device: &LogicalDevice) {
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
