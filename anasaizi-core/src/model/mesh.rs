use crate::{
    vulkan::{IndexBuffer, VertexBuffer},
};
use nalgebra::Matrix4;

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
}
