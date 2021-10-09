use crate::engine::gltf::root::Root;
use crate::math::Matrix4;

pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<usize>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            name: None,
            nodes: vec![],
        }
    }
}

impl Scene {
    pub fn from_gltf(g_scene: &gltf::Scene<'_>, root: &mut Root) -> Scene {
        let mut scene = Scene {
            name: g_scene.name().map(|s| s.to_owned()),
            ..Default::default()
        };
        scene.nodes = g_scene.nodes()
            .map(|g_node| g_node.index())
            .collect();

        // propagate transforms
        let root_transform = Matrix4::identity();
        for node_id in &scene.nodes {
            let node = root.unsafe_get_node_mut(*node_id);
            node.update_transform(root, &root_transform);
        }

        scene
    }
}
