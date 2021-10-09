use crate::math::Vertex;
use nalgebra::{Vector2, Vector3, Vector4};
use std::path::Path;
use tobj::LoadOptions;

/// Wavefront .obj file loader.
pub struct Object;

impl Object {
    /// Loads an `.obj` file from the given path and returns its contents.
    ///
    /// Object file must contain:
    /// - Position (x, y, z)
    /// - Texture Coordinates (u, v)
    pub fn load_model(model_path: &Path) -> (Vec<Vertex>, Vec<u32>) {
        let model_obj = tobj::load_obj(
            model_path,
            &LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load model object!");

        let mut vertices = vec![];
        let mut indices = vec![];

        let (models, _) = model_obj;
        for m in models.iter() {
            let mesh = &m.mesh;

            if mesh.texcoords.len() == 0 {
                panic!("Missing texture coordinate for the model.")
            }

            let total_vertices_count = mesh.positions.len() / 3;
            for i in 0..total_vertices_count {
                let vertex = Vertex {
                    pos: Vector3::new(
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                    ),
                    color: Vector4::new(1.0, 1.0, 1.0, 0.0),
                    tex_coord: Vector2::new(mesh.texcoords[(i * 2)], mesh.texcoords[(i * 2 + 1)]),
                    normal: Vector3::new(
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ),
                    tangent: Vector3::zeros(),
                    bitangent: Vector3::zeros()
                };
                vertices.push(vertex);
            }

            let mut i = 0;
            while i < indices.len() {
                assert!(i+3 <= indices.len());
                let mut a = vertices[mesh.indices[i*3] as usize];
                let mut b = vertices[mesh.indices[i*3+1]as usize];
                let mut c = vertices[mesh.indices[i*3+2]as usize];

                let edge1 = b.pos - a.pos;
                let edge2 = c.pos - a.pos;

                let delta_uv1 = b.tex_coord - a.tex_coord;
                let delta_uv2 = c.tex_coord - a.tex_coord;

                let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

                let mut tangent1 = Vector3::zeros();
                tangent1.x = f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x);
                tangent1.y = f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y);
                tangent1.z = f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z);
                tangent1.normalize();

                let mut bitangent1 = Vector3::zeros();
                bitangent1.x = f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x);
                bitangent1.y = f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y);
                bitangent1.z = f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z);
                bitangent1.normalize();

                vertices[mesh.indices[i*3]as usize].tangent = tangent1;
                vertices[mesh.indices[i*3+1]as usize].tangent = tangent1;
                vertices[mesh.indices[i*3+2]as usize].tangent = tangent1;

                vertices[mesh.indices[i*3]as usize].bitangent = bitangent1;
                vertices[mesh.indices[i*3+1]as usize].bitangent = bitangent1;
                vertices[mesh.indices[i*3+2]as usize].bitangent = bitangent1;

                i += 3;
            }

            indices = mesh.indices.clone();
        }

        (vertices, indices)
    }
}
