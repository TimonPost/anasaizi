use crate::engine::{
    image::Texture, resources::texture_collection::TextureCollection, RenderContext,
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct StoredTexture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub path: String,
    pub id: String,
    texture: Option<Texture>,
}

impl StoredTexture {
    pub fn new(
        data: Vec<u8>,
        width: u32,
        height: u32,
        path: String,
        id: String,
        texture: Option<Texture>,
    ) -> StoredTexture {
        StoredTexture {
            data,
            width,
            height,
            path,
            id,
            texture,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.texture.is_some()
    }

    pub fn texture(&self) -> &Texture {
        &self.texture.as_ref().unwrap()
    }

    pub fn owned_texture(&self) -> Texture {
        self.texture.as_ref().unwrap().clone()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn load_textures(&mut self, render_context: &RenderContext) {
        if !self.data.is_empty() {
            self.texture = Some(Texture::from_bytes(
                &render_context,
                &self.data,
                self.width,
                self.height,
            ));
        }
    }
}

pub struct TextureStorage {
    textures: HashMap<String, TextureCollection>,
}

impl TextureStorage {
    pub fn new() -> TextureStorage {
        TextureStorage {
            textures: HashMap::new(),
        }
    }

    pub fn get_category(&self, id: &'static str) -> &TextureCollection {
        self.textures
            .get(id)
            .expect(&format!("Texture with id: {} could not be found.", id))
    }

    pub fn query(&self, id: &'static str) -> &StoredTexture {
        let texture_id = TextureId::new(id);

        let category = self
            .textures
            .get(&texture_id.category())
            .expect(&format!("Texture with id: {} could not be found.", id));

        category.get(texture_id.texture_ref())
    }

    pub fn get_mut(&mut self, id: &str) -> &mut TextureCollection {
        self.textures
            .get_mut(id)
            .expect(&format!("Texture with id: {} could not be found.", id))
    }

    pub fn insert(&mut self, texture_id: TextureId, texture: StoredTexture) {
        let entry = self
            .textures
            .entry(texture_id.category())
            .or_insert(TextureCollection::new());
        entry.insert(texture_id.texture_ref(), texture);
    }

    pub fn load_textures(&mut self, render_context: &RenderContext) {
        for texture in self.textures.iter_mut() {
            texture.1.load_textures(render_context);
        }
    }
}

#[derive(Clone)]
pub struct TextureId(String, String);

impl TextureId {
    pub fn new(id: &str) -> TextureId {
        let split = id.split(".").collect::<Vec<&str>>();
        TextureId(split[0].to_string(), split[1].to_string())
    }

    pub fn category_ref(&self) -> &str {
        &self.0
    }

    pub fn texture_ref(&self) -> &str {
        &self.1
    }

    pub fn category(&self) -> String {
        self.0.clone()
    }

    pub fn texture(&self) -> String {
        self.1.clone()
    }
}
