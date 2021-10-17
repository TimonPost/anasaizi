use crate::{
    engine::{
        gltf::{mappers::ImportData, mesh::GLTFMesh, node::GLTFNode},
        GLTFMaterial, GpuMeshMemory, RenderContext, Transform,
    },
    vulkan::{ShaderFlags, ShaderSet},
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    rc::Rc,
};

#[derive(Default)]
pub struct GLTFRoot {
    pub nodes: Vec<GLTFNode>,
    pub meshes: Vec<Rc<GLTFMesh>>,

    pub materials: Vec<(usize, GLTFMaterial)>,

    pub textures: Vec<crate::engine::image::Texture>,
    pub texture_paths: HashSet<String>,
    pub shaders: HashMap<ShaderFlags, ShaderSet>,
    pub entities: HashMap<ShaderFlags, Vec<(GpuMeshMemory, Transform, GLTFMaterial)>>,
}

impl GLTFRoot {
    pub(crate) fn add_entity(
        &mut self,
        flags: ShaderFlags,
        memory: GpuMeshMemory,
        transform: Transform,
        material: GLTFMaterial,
    ) {
        let entry = self.entities.entry(flags).or_insert(Vec::new());
        entry.push((memory, transform, material))
    }

    pub(crate) fn add_texture(&mut self, path: String) -> usize {
        let id = self.texture_paths.len();
        self.texture_paths.insert(path);
        id
    }
}

impl GLTFRoot {
    pub fn from_gltf(
        render_context: &mut RenderContext,
        imp: &ImportData,
        base_path: &Path,
    ) -> Self {
        let mut root = GLTFRoot::default();
        let nodes = imp
            .doc
            .nodes()
            .map(|g_node| GLTFNode::from_gltf(render_context, &g_node, &mut root, imp, base_path))
            .collect();

        root.nodes = nodes;
        root
    }

    /// Get a mutable reference to a node without borrowing `Self` or `Self::nodes`.
    /// Safe for tree traversal (visiting each node ONCE and NOT keeping a reference)
    /// as long as the gltf is valid, i.e. the scene actually is a tree.
    pub fn unsafe_get_node_mut(&mut self, index: usize) -> &'static mut GLTFNode {
        unsafe { &mut *(&mut self.nodes[index] as *mut GLTFNode) }
    }
}
