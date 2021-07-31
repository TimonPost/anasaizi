use crate::{
    math::Vertex,
    reexports::imgui::DrawData,
    vulkan::{
        create_buffer, CommandPool, IndexBuffer, Instance, LogicalDevice, Queue, VertexBuffer,
    },
};
use ash::{version::DeviceV1_0, vk};
use nalgebra::Matrix4;
use std::mem::size_of;

pub struct Mesh {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    model_transform: nalgebra::Matrix4<f32>,
}

impl Mesh {
    pub fn new(vertex_buffer: VertexBuffer, index_buffer: IndexBuffer) -> Mesh {
        Mesh {
            vertex_buffer,
            model_transform: Matrix4::default(),
            index_buffer,
        }
    }

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

    pub fn get_vertices_and_indices(draw_data: &DrawData) -> (Vec<Vertex>, Vec<u32>) {
        // load vertexes
        let vertex_count = draw_data.total_vtx_count as usize;
        let mut vertices: Vec<Vertex> = Vec::with_capacity(vertex_count);
        let mut count = 0;
        let mut count1 = 0;
        for draw_list in draw_data.draw_lists() {
            let vertexes = draw_list
                .vtx_buffer()
                .iter()
                .map(|vertex| {
                    if (vertex.col[0] == 255 && vertex.col[1] == 255 && vertex.col[2] == 255) {
                        count += 1;
                    } else {
                        count1 += 1;
                    }
                    Vertex {
                        pos: nalgebra::Vector3::new(vertex.pos[0], vertex.pos[1], 0.0),
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

    pub fn update_from_draw_data(
        &mut self,
        instance: &Instance,
        device: &LogicalDevice,
        submit_queue: &Queue,
        command_pool: &CommandPool,
        draw_data: &DrawData,
    ) {
        let (vertices, indices) = Self::get_vertices_and_indices(draw_data);

        if draw_data.total_vtx_count as usize > self.vertex_buffer.vertices_count() {
            println!("Resizing vertex buffers");
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
            self.vertex_buffer.update_buffer_content(&device, &vertices);
        }

        if draw_data.total_idx_count as usize > self.index_buffer.indices_count() {
            println!("Resizing index buffers");
            self.index_buffer.destroy(device);

            let index_buffer =
                IndexBuffer::create(instance, device, &indices, submit_queue, command_pool);
            self.index_buffer = index_buffer;
        } else {
            self.index_buffer.update_buffer_content(&device, &indices);
        }
    }

    pub fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &IndexBuffer {
        &self.index_buffer
    }

    pub fn vertices_count(&self) -> usize {
        self.vertex_buffer.vertices_count()
    }

    pub fn indices_count(&self) -> usize {
        self.index_buffer.indices_count()
    }

    pub fn destroy(&self, device: &LogicalDevice) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
    }
}
