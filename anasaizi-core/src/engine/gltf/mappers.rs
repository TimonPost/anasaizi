use crate::engine::{
    gltf::{gltf_buffer_loader::GltfBufferStorage, gltf_texture_loader::GltfTextureStorage},
    Camera,
};
use gltf::{camera::Projection, Document};

pub struct ImportData {
    pub buffer_storage: GltfBufferStorage,
    pub texture_storage: GltfTextureStorage,
    pub doc: Document,
}

pub fn camara_from_gltf(gltf_camera: &gltf::Camera) -> Camera {
    let camera = match gltf_camera.projection() {
        Projection::Orthographic(_) => {
            panic!("Gltf orthographic camara not supported.")
        }
        Projection::Perspective(perspective) => Camera::new(
            perspective.aspect_ratio().unwrap() as f32,
            perspective.yfov(),
            perspective.znear(),
            perspective.zfar().unwrap() as f32,
        ),
    };

    camera
}
