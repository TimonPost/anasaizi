use std::path::Path;
use crate::engine::gltf::mappers::ImportData;
use crate::engine::gltf::root::Root;
use crate::engine::gltf::primitive::Primitive;
use crate::math::{Matrix4, Vector3};
use crate::engine::{RenderContext, Transform};

#[derive(Clone)]
pub struct Mesh {
    pub index: usize, // glTF index
    pub primitives: Vec<Primitive>,
    pub name: Option<String>,
}

impl Mesh {
    pub fn from_gltf(
        render_context: &mut RenderContext,
        g_mesh: &gltf::Mesh<'_>,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path,
        transform: Transform
    ) -> Mesh {
        let primitives: Vec<Primitive> = g_mesh.primitives()
            .enumerate()
            .map(|(i, g_prim)| {
                Primitive::from_gltf(render_context,&g_prim, i, g_mesh.index(), root, imp, base_path, transform)
            })
            .collect();

        Mesh {
            index: g_mesh.index(),
            primitives,
            name: g_mesh.name().map(|s| s.into()),
        }
    }
}