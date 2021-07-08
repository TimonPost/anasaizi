use crate::vulkan::VertexBuffer;
use crate::math::Vertex;
use nalgebra::Matrix4;

pub struct Mesh {
    vertex_buffer: VertexBuffer,
    model_transform: nalgebra::Matrix4<f32>,
    vertices: Vec<Vertex>
}

impl Mesh {
    pub fn new(vertex_buffer: VertexBuffer, vertices: Vec<Vertex>) -> Mesh {
        Mesh {
            vertex_buffer,
            model_transform: Matrix4::default(),
            vertices
        }
    }

    pub fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    pub fn vertices_count(&self) -> usize {
        self.vertices.len()
    }
}