use crate::engine::gltf::mappers::ImportData;
use std::path::Path;
use crate::engine::gltf::root::Root;
use crate::engine::gltf::scene::Scene;
use crate::engine::RenderContext;
use gltf::{Gltf, Error};
use crate::libs::tokio::{fs, io};
use tokio::io::AsyncReadExt;
use gltf::image::Source;
use image::{ImageFormat, GenericImageView};
use crate::engine::resources::{TextureLoader, TextureStorage, StoredTexture, TextureId};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::libs::tokio::task::JoinHandle;
use crate::engine::image::Texture;

pub async fn load_gltf_scene(mut render_context: RenderContext, source: &str, scene_index: usize) -> (Root, Scene) {
    let base = Path::new(&source).parent().unwrap_or(Path::new("./"));
    let gltf_file = Path::new(source);

    let file = fs::File::open(gltf_file).await.unwrap();
    let mut reader = io::BufReader::new(file);

    let mut bytes = Vec::new();
    let size = reader.read_to_end(&mut bytes).await.unwrap();

    let mut gltf = Gltf::from_slice(&bytes).unwrap();

    let mut texture_loader = GltfTextureLoader::new();
    let mut buffer_loader = GltfBufferLoader::new();

    let buffer_storage = load_buffers(&mut gltf, buffer_loader,base).await;
    let texture_storage = load_images(&mut gltf, texture_loader,base).await;

    let data = ImportData { buffer_storage, texture_storage, doc: gltf.document };

    let base_path = Path::new(source);
    let mut root = Root::from_gltf(&mut render_context, &data, base_path);
    let scene = Scene::from_gltf(&data.doc.scenes().nth(scene_index).unwrap(), &mut root);

    root.textures = data.texture_storage.to_vulkan_textures(&mut render_context);
    (root, scene)
}

async fn load_buffers(gltf: &mut Gltf, mut buffer_loader: GltfBufferLoader, base: &Path) -> GltfBufferStorage {
    //let raw_buffers = gltf.blob.as_ref().unwrap();

    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
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

async fn load_images(gltf: &mut Gltf, mut texture_loader: GltfTextureLoader, base: &Path) -> GltfTextureStorage {
    let mut id = "";

    for image in gltf.images() {
        match image.source() {
            gltf::image::Source::View { view, mime_type } => {
                // texture_loader.load_buffer(view.index(), view.stride(), view.offset(), view.length(), view.buffer(), &image.index().to_string(), false);
                //
                // crate::engine::image::Texture::from_bytes(render_context, &image_data, image_object.width(), image_object.height())
                panic!("not supported")
            }
            gltf::image::Source::Uri { mime_type, uri } => {
                texture_loader.load_path(base.join(uri).to_str().unwrap(), image.index());
            }
        };
    }

    texture_loader.wait_loading().await
}

pub struct GltfTextureStorage {
    buffers: HashMap<usize, (usize, StoredTexture)>,
}

impl GltfTextureStorage {
    pub fn new() -> GltfTextureStorage {
        GltfTextureStorage {
            buffers: HashMap::new()
        }
    }

    pub fn add_texture(&mut self, index: usize, texture: StoredTexture) {
        self.buffers.insert(index, (self.buffers.len(), texture));
    }

    pub fn at(&self, index: usize) -> &(usize, StoredTexture) {
        &self.buffers[&index]
    }

    pub fn to_vulkan_textures(mut self, render_context: &mut RenderContext) -> Vec<Texture> {
        let mut values = self.buffers.values_mut()
            .collect::<Vec<&mut (usize, StoredTexture)>>();

        values.sort_by(|a, b|a.0.cmp(&b.0));

        values.iter_mut()
            .map(|x| {
                x.1.load_textures(render_context);
                x.1.texture().clone()
            }).collect()
    }
}

pub struct GltfTextureLoader {
    storage: Arc<Mutex<GltfTextureStorage>>,
    load_tasks: Vec<JoinHandle<()>>,
}

impl GltfTextureLoader {
    pub fn new() -> GltfTextureLoader {
        GltfTextureLoader {
            storage: Arc::new(Mutex::new(GltfTextureStorage::new())),
            load_tasks: Vec::new(),
        }
    }

    pub fn load_path(&mut self, path: &str, id: usize) {
        let task = tokio::spawn(load_image_path(
            path.to_string(),
            id,
            self.storage.clone(),
            false
        ));
        self.load_tasks.push(task);
    }

    pub async fn wait_loading(self) -> GltfTextureStorage {
        futures::future::join_all(self.load_tasks).await;

        if let Ok(lock) = Arc::try_unwrap(self.storage) {
            let mut lock = lock.into_inner().expect("Mutex cannot be locked");
            return lock;
        }
        panic!("Could not acquire storage inner value");
    }
}

pub struct GltfBufferStorage {
    buffers: HashMap<usize, Vec<u8>>,
}

impl GltfBufferStorage {
    pub fn new() -> GltfBufferStorage {
        GltfBufferStorage {
            buffers: HashMap::new()
        }
    }

    pub fn add_buffer(&mut self, index: usize, buffer: Vec<u8>) {
        self.buffers.insert(index, buffer);
    }

    pub fn at(&self, index: usize) -> &Vec<u8> {
        &self.buffers[&index]
    }
}

pub struct GltfBufferLoader {
    storage: Arc<Mutex<GltfBufferStorage>>,
    load_tasks: Vec<JoinHandle<()>>,
}

impl GltfBufferLoader {
    pub fn new() -> GltfBufferLoader {
        GltfBufferLoader {
            storage: Arc::new(Mutex::new(GltfBufferStorage::new())),
            load_tasks: Vec::new(),
        }
    }

    pub fn load_path(&mut self, path: &str, id: usize) {
        let task = tokio::spawn(load_buffer_file(
            path.to_string(),
            id,
            self.storage.clone(),
        ));
        self.load_tasks.push(task);
    }

    pub fn load_buffer(&mut self, buffer: &[u8], id: usize) {
        let mut lock = self.storage.lock().unwrap();
        lock.add_buffer(id ,buffer.to_vec());
    }

    pub async fn wait_loading(self) -> GltfBufferStorage {
        futures::future::join_all(self.load_tasks).await;

        if let Ok(lock) = Arc::try_unwrap(self.storage) {
            let mut lock = lock.into_inner().expect("Mutex cannot be locked");
            return lock;
        }
        panic!("Could not acquire storage inner value");
    }
}

async fn load_buffer_file(path: String, index: usize, storage: Arc<Mutex<GltfBufferStorage>>) {
    let base = Path::new(&path);
    let file = fs::File::open(base).await.unwrap();
    let mut reader = io::BufReader::new(file);

    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await.unwrap();

    let mut lock = storage.lock().unwrap();
    lock.add_buffer(index ,bytes);
}


pub async fn load_image_path(
    path: String,
    id: usize,
    image_results: Arc<Mutex<GltfTextureStorage>>,
    debug: bool,
) {
    let contents = if !debug {
        tokio::fs::read(path.clone()).await.unwrap()
    } else {
        tokio::fs::read("assets/textures/debug/albedo.jpg")
            .await
            .unwrap()
    };

    let mut image_object = image::load_from_memory(&contents).unwrap();
    image_object = image_object.flipv();

    let (image_width, image_height) = (image_object.width(), image_object.height());

    let image_data = match &image_object {
        image::DynamicImage::ImageLuma8(_)
        | image::DynamicImage::ImageBgr8(_)
        | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba8().into_raw(),
        image::DynamicImage::ImageLumaA8(_)
        | image::DynamicImage::ImageBgra8(_)
        | image::DynamicImage::ImageRgba8(_) => image_object.to_bytes(),
        _ => {
            panic!("Invalid image format, image should be rgba compatible");
        }
    };

    let stored_texture = StoredTexture::new(
        image_data,
        image_width,
        image_height,
        path,
        id.to_string(),
        None,
    );

    let mut lock = image_results.lock().unwrap();
    lock.add_texture(id, stored_texture);
}