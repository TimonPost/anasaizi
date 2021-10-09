use crate::engine::{Camera, PBRMeshPushConstants, RenderContext};
use gltf::camera::Projection;
use gltf::json::{Root, Path};
use std::rc::Rc;
use gltf::image::Source;
use image::{ImageFormat, GenericImageView};
use std::{fs, io};
use crate::engine::gltf::loader::{GltfTextureStorage, GltfBufferStorage};
use gltf::Document;

pub struct ImportData {
    pub buffer_storage: GltfBufferStorage,
    pub texture_storage: GltfTextureStorage,
    pub doc: Document
}

pub fn camara_from_gltf(gltf_camera: &gltf::Camera) -> Camera {
    let camera = match gltf_camera.projection() {
        Projection::Orthographic(_) => { panic!("Orthographic Not supported at the moment.")}
        Projection::Perspective(perspective) => {
            Camera::new(perspective.aspect_ratio().unwrap() as f32, perspective.yfov(), perspective.znear(), perspective.zfar().unwrap() as f32)
        }
    };

    camera
}