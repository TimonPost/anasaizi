use crate::engine::{image::Texture, resources::StoredTexture, RenderContext};
use image::GenericImageView;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

pub struct GltfTextureStorage {
    buffers: HashMap<usize, (usize, StoredTexture)>,
}

impl GltfTextureStorage {
    pub fn new() -> GltfTextureStorage {
        GltfTextureStorage {
            buffers: HashMap::new(),
        }
    }

    pub fn add_texture(&mut self, index: usize, texture: StoredTexture) {
        self.buffers.insert(index, (self.buffers.len(), texture));
    }

    pub fn at(&self, index: usize) -> &(usize, StoredTexture) {
        &self.buffers[&index]
    }

    pub fn to_vulkan_textures(mut self, render_context: &mut RenderContext) -> Vec<Texture> {
        let mut values = self
            .buffers
            .values_mut()
            .collect::<Vec<&mut (usize, StoredTexture)>>();

        values.sort_by(|a, b| a.0.cmp(&b.0));

        values
            .iter_mut()
            .map(|x| {
                x.1.load_textures(render_context);
                x.1.texture().clone()
            })
            .collect()
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
            false,
        ));
        self.load_tasks.push(task);
    }

    pub async fn wait_loading(self) -> GltfTextureStorage {
        futures::future::join_all(self.load_tasks).await;

        if let Ok(lock) = Arc::try_unwrap(self.storage) {
            let lock = lock.into_inner().expect("Mutex cannot be locked");
            return lock;
        }
        panic!("Could not acquire storage inner value");
    }
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
