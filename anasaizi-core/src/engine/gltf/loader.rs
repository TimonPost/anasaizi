use crate::{
    engine::{
        gltf::{
            gltf_buffer_loader::{GltfBufferLoader, GltfBufferStorage},
            gltf_texture_loader::{GltfTextureLoader, GltfTextureStorage},
            mappers::ImportData,
            root::GLTFRoot,
            scene::GLTFScene,
        },
        RenderContext,
    },
    libs::tokio::{fs, io},
};
use gltf::Gltf;

use std::path::Path;
use tokio::io::AsyncReadExt;

pub async fn load_gltf_scene(
    mut render_context: RenderContext,
    source: &str,
    scene_index: usize,
) -> (GLTFRoot, GLTFScene) {
    let base = Path::new(&source).parent().unwrap_or(Path::new("./"));
    let gltf_file = Path::new(source);

    let file = fs::File::open(gltf_file).await.unwrap();
    let mut reader = io::BufReader::new(file);

    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await.unwrap();

    let mut gltf = Gltf::from_slice(&bytes).unwrap();

    let texture_loader = GltfTextureLoader::new();
    let buffer_loader = GltfBufferLoader::new();

    let buffer_storage = load_buffers(&mut gltf, buffer_loader, base).await;
    let texture_storage = load_images(&mut gltf, texture_loader, base).await;

    let data = ImportData {
        buffer_storage,
        texture_storage,
        doc: gltf.document,
    };

    let base_path = Path::new(source);
    let mut root = GLTFRoot::from_gltf(&mut render_context, &data, base_path);
    let scene = GLTFScene::from_gltf(&data.doc.scenes().nth(scene_index).unwrap(), &mut root);

    root.textures = data.texture_storage.to_vulkan_textures(&mut render_context);
    (root, scene)
}

async fn load_buffers(
    gltf: &mut Gltf,
    mut buffer_loader: GltfBufferLoader,
    base: &Path,
) -> GltfBufferStorage {
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                panic!("GLTF buffer binary source is not supported.")
                // let begin = buffer.offset();
                // let end = begin + buffer.length();
                // let data: &[u8] = &raw_buffers[begin..end];
                //
                // buffer_loader.load_buffer(data, buffer.index());
            }
            gltf::buffer::Source::Uri(path) => {
                buffer_loader.load_path(base.join(path).to_str().unwrap(), buffer.index());
            }
        }
    }

    buffer_loader.wait_loading().await
}

async fn load_images(
    gltf: &mut Gltf,
    mut texture_loader: GltfTextureLoader,
    base: &Path,
) -> GltfTextureStorage {
    for image in gltf.images() {
        match image.source() {
            gltf::image::Source::View {
                view: _,
                mime_type: _,
            } => {
                // texture_loader.load_buffer(view.index(), view.stride(), view.offset(), view.length(), view.buffer(), &image.index().to_string(), false);
                //
                // crate::engine::image::Texture::from_bytes(render_context, &image_data, image_object.width(), image_object.height())
                panic!("GLTF image view source is not supported.")
            }
            gltf::image::Source::Uri { mime_type: _, uri } => {
                texture_loader.load_path(base.join(uri).to_str().unwrap(), image.index());
            }
        };
    }

    texture_loader.wait_loading().await
}
