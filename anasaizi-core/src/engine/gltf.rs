use gltf::Gltf;

pub struct GltfObject {

}

impl GltfObject {
    pub fn new(path: &str) -> GltfObject {
        let gltf = Gltf::open(path)?;
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                println!(
                    "Node #{} has {} children",
                    node.index(),
                    node.children().count(),
                );
            }
        }

        let (document, buffers, images) = gltf::import("examples/Box.gltf")?;
        document.meshes()[0]


        GltfObject {

        }
    }
}