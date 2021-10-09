use crate::vulkan::{ShaderSet, ShaderFlags};
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use crate::engine::{PBRMeshPushConstants, RenderContext, GpuMeshMemory, Transform, GlTFPBRMeshPushConstants};
use crate::engine::gltf::mappers::ImportData;
use crate::engine::gltf::node::Node;
use std::path::Path;
use crate::engine::gltf::mesh::Mesh;
use crate::engine::image::Texture;

#[derive(Default)]
pub struct Root {
    pub nodes: Vec<Node>,
    pub meshes: Vec<Rc<Mesh>>,

    pub materials: Vec<(usize, GlTFPBRMeshPushConstants)>,

    pub textures: Vec<crate::engine::image::Texture>,
    pub texture_paths: HashSet<String>,
    pub shaders: HashMap<ShaderFlags, ShaderSet>,
    pub entities: HashMap<ShaderFlags, Vec<(GpuMeshMemory, Transform, GlTFPBRMeshPushConstants)>>
}

impl Root {
    pub(crate) fn add_entity(&mut self, flags: ShaderFlags, memory: GpuMeshMemory, transform: Transform, material: GlTFPBRMeshPushConstants) {
        let entry = self.entities.entry(flags).or_insert(Vec::new());
        entry.push((memory, transform, material))
    }

    pub(crate) fn add_texture(&mut self, path: String) -> usize {
        let id = self.texture_paths.len();
        self.texture_paths.insert(path);
        id
    }
}

impl Root {
    pub fn from_gltf(render_context: &mut RenderContext, imp: &ImportData, base_path: &Path) -> Self {
        let mut root = Root::default();
        let nodes = imp.doc.nodes()
            .map(|g_node| Node::from_gltf(render_context,&g_node, &mut root, imp, base_path))
            .collect();

        root.nodes = nodes;
        root
    }

    /// Get a mutable reference to a node without borrowing `Self` or `Self::nodes`.
    /// Safe for tree traversal (visiting each node ONCE and NOT keeping a reference)
    /// as long as the gltf is valid, i.e. the scene actually is a tree.
    pub fn unsafe_get_node_mut(&mut self, index: usize) ->&'static mut Node {
        unsafe {
            &mut *(&mut self.nodes[index] as *mut Node)
        }
    }
}