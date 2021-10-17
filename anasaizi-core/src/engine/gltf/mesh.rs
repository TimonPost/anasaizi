use crate::engine::{
    gltf::{mappers::ImportData, primitive::GLTFPrimitive, root::GLTFRoot},
    RenderContext, Transform,
};
use std::path::Path;

#[derive(Clone)]
pub struct GLTFMesh {
    pub index: usize, // glTF index
    pub primitives: Vec<GLTFPrimitive>,
    pub name: Option<String>,
}

impl GLTFMesh {
    pub fn from_gltf(
        render_context: &mut RenderContext,
        g_mesh: &gltf::Mesh<'_>,
        root: &mut GLTFRoot,
        imp: &ImportData,
        base_path: &Path,
        transform: Transform,
    ) -> GLTFMesh {
        let primitives: Vec<GLTFPrimitive> = g_mesh
            .primitives()
            .enumerate()
            .map(|(i, g_prim)| {
                GLTFPrimitive::from_gltf(
                    render_context,
                    &g_prim,
                    i,
                    g_mesh.index(),
                    root,
                    imp,
                    base_path,
                    transform,
                )
            })
            .collect();

        GLTFMesh {
            index: g_mesh.index(),
            primitives,
            name: g_mesh.name().map(|s| s.into()),
        }
    }
}
