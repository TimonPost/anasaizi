use crate::math::{Matrix4, Vector3, Quaternion};
use crate::engine::gltf::root::Root;
use crate::engine::gltf::mappers::{ImportData, camara_from_gltf};
use crate::engine::{GpuMeshMemory, Camera, Transform, RenderContext};
use std::rc::Rc;
use std::path::Path;
use crate::engine::gltf::mesh::Mesh;

pub struct Node {
    pub index: usize, // glTF index
    pub children: Vec<usize>,

    pub mesh: Option<Rc<Mesh>>,
    pub transform: Transform,

    pub name: Option<String>,

    pub parent_transform: Matrix4
}

impl Node {
    pub fn from_gltf(
        render_context: &mut RenderContext,
        g_node: &gltf::Node<'_>,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path
    ) -> Node {
        let (trans, rot, scale) = g_node.transform().decomposed();

        let transform = Transform::new(1.0)
            .with_rotation(Vector3::new(rot[0], rot[1], rot[2]))
            .with_translate(Vector3::new(trans[0], trans[1], trans[2]))
            .with_scale(Vector3::new(scale[0], scale[1], scale[2]));

        let mut mesh: Option<Rc<Mesh>> = None;

        if let Some(g_mesh) = g_node.mesh() {
            if let Some(existing_mesh) = root.meshes.iter().find(|mesh| (**mesh).index == g_mesh.index()) {
                mesh = Some(existing_mesh.clone());
            }

            if mesh.is_none() { // not using else due to borrow-checking madness
                mesh = Some(Rc::new(Mesh::from_gltf(render_context, &g_mesh, root, imp, base_path, transform)));
                root.meshes.push(mesh.as_ref().unwrap().clone());
            }
        }

        let children: Vec<_> = g_node.children()
            .map(|g_node| g_node.index())
            .collect();

        Node {
            index: g_node.index(),
            children,
            mesh,

            transform,

            name: g_node.name().map(|s| s.into()),

            parent_transform: Matrix4::identity()
        }
    }

    pub fn update_transform(&mut self, root: &mut Root, parent_transform: &Matrix4) {
        self.parent_transform = *parent_transform;

        let transform = self.parent_transform * self.transform.model_transform();
        for node_id in &self.children {
            let node = root.unsafe_get_node_mut(*node_id);
            node.update_transform(root, &transform);
        }
    }
}
