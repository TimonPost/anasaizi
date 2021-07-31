use crate::math::Vertex;
use nalgebra::{Vector2, Vector3, Vector4};
use std::path::Path;
use tobj::LoadOptions;

pub struct Object {}

impl Object {
    pub fn load_model(model_path: &Path) -> (Vec<Vertex>, Vec<u32>) {
        let model_obj = tobj::load_obj(model_path, &LoadOptions::default())
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
                    tex_coord: Vector2::new(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]),
                };
                vertices.push(vertex);
            }

            indices = mesh.indices.clone();
        }

        (vertices, indices)
    }
}
