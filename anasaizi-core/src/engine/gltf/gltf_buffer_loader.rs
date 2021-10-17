use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{fs, io, io::AsyncReadExt, task::JoinHandle};

pub struct GltfBufferStorage {
    buffers: HashMap<usize, Vec<u8>>,
}

impl GltfBufferStorage {
    pub fn new() -> GltfBufferStorage {
        GltfBufferStorage {
            buffers: HashMap::new(),
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
        let task = tokio::spawn(load_buffer_file(path.to_string(), id, self.storage.clone()));
        self.load_tasks.push(task);
    }

    pub fn load_buffer(&mut self, buffer: &[u8], id: usize) {
        let mut lock = self.storage.lock().unwrap();
        lock.add_buffer(id, buffer.to_vec());
    }

    pub async fn wait_loading(self) -> GltfBufferStorage {
        futures::future::join_all(self.load_tasks).await;

        if let Ok(lock) = Arc::try_unwrap(self.storage) {
            let lock = lock.into_inner().expect("Mutex cannot be locked");
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
    lock.add_buffer(index, bytes);
}
