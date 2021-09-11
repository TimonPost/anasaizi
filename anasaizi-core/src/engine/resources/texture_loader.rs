use crate::engine::{
    resources::{load_image, TextureStorage},
    RenderContext,
};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

pub struct TextureLoader {
    storage: Arc<Mutex<TextureStorage>>,
    load_tasks: Vec<JoinHandle<()>>,
    render_context: Arc<RenderContext>,
    debug: bool,
}

impl TextureLoader {
    pub fn new(render_context: RenderContext) -> TextureLoader {
        TextureLoader {
            storage: Arc::new(Mutex::new(TextureStorage::new())),
            load_tasks: Vec::new(),
            render_context: Arc::new(render_context),
            debug: false,
        }
    }

    pub fn load(&mut self, path: &'static str, id: &'static str, debug: bool) {
        let task = tokio::spawn(load_image(
            path,
            id,
            self.storage.clone(),
            self.debug && debug,
        ));
        self.load_tasks.push(task);
    }

    pub fn set_debug(&mut self, value: bool) {
        self.debug = value;
    }

    pub async fn wait_loading(self) -> TextureStorage {
        futures::future::join_all(self.load_tasks).await;

        if let Ok(lock) = Arc::try_unwrap(self.storage) {
            let mut lock = lock.into_inner().expect("Mutex cannot be locked");
            lock.load_textures(&*self.render_context);
            return lock;
        }
        panic!("Could not acquire storage inner value");
    }
}
