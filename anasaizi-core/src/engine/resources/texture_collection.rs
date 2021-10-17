use crate::engine::{resources::StoredTexture, RenderContext};
use std::collections::HashMap;

pub struct TextureCollection {
    textures: HashMap<String, StoredTexture>,
}

impl TextureCollection {
    pub fn new() -> TextureCollection {
        TextureCollection {
            textures: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: &str, texture: StoredTexture) {
        self.textures.insert(id.to_string(), texture);
    }

    pub fn get(&self, id: &str) -> &StoredTexture {
        self.textures
            .get(id)
            .expect(&format!("Texture with id: {} could not be found.", id))
    }

    pub fn load_textures(&mut self, render_context: &RenderContext) {
        for texture in self.textures.iter_mut() {
            texture.1.load_textures(render_context);
        }
    }
}
