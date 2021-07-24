use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    collections::LinkedList,
    fmt::Write,
    fs,
    fs::{File, OpenOptions},
    io,
    io::BufRead,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

lazy_static! {
    /// This is an example for using doc comment attributes
    pub static ref PROFILER: Mutex<Profiler> = Mutex::new(Profiler::new());
}

#[macro_export]
macro_rules! profile_fn {
    ($profile:expr, $to_profile:expr) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let mut name = type_name_of(f);
        // Find and cut the rest of the path
        name = match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        };

        let mut lock = PROFILER.lock().unwrap();
        let profile_data = lock.start_profile(format!("{} - {}", name, $profile));
        drop(lock);

        let result = $to_profile;

        profile_data.end_profile();

        result
    }};
}

pub struct Batch {
    records: LinkedList<ProfileObject>,
    max_size: usize,
    output: &'static str,
    start_profile_time: Instant,
}

impl Batch {
    pub fn new(output: &'static str, start_profile_time: Instant) -> Batch {
        Batch {
            output,
            max_size: 0,
            records: LinkedList::new(),
            start_profile_time,
        }
    }

    pub fn add(&mut self, profile_object: ProfileObject) {
        self.records.push_back(profile_object)
    }

    pub fn flush(&mut self) {
        let mut result = String::new();

        while let Some(record) = self.records.pop_back() {
            if let Some(start_of_record) = record
                .start_time
                .checked_duration_since(self.start_profile_time)
            {
                write!(
                    result,
                    "\n{},{},{}",
                    record.profile_fn,
                    record.duration.as_micros(),
                    start_of_record.as_micros()
                );
            }
        }

        Self::write_to_file(self.output, &result);
    }

    pub fn can_flush(&self) -> bool {
        self.records.len() > self.max_size
    }

    pub fn write_to_file(path: &str, output: &str) {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        {
            use std::io::Write as OtherWrite;
            file.write_all(output.as_bytes()).unwrap();
        }
    }
}

pub struct Profiler {
    channel_tx: Sender<ProfileObject>,
    channel_rx: Option<Receiver<ProfileObject>>,
    output_path: &'static str,
    is_profiling: Arc<AtomicBool>,
}

impl Profiler {
    pub fn new() -> Profiler {
        let (tx, rx) = channel();

        Profiler {
            channel_tx: tx,
            channel_rx: Some(rx),
            output_path: "output.json",
            is_profiling: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_profile(&self, log: String) -> ProfileObject {
        ProfileObject::new(log)
    }

    pub fn end_profile(&mut self, mut profile_object: ProfileObject) {
        let elapsed = profile_object.start_time.elapsed();
        profile_object.duration = elapsed;
        self.channel_tx.send(profile_object);
    }

    pub fn start_session(&mut self) {
        self.is_profiling
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);

        self.remove_file();

        let start_time = get_current_time_ns();

        let rx = self.channel_rx.take().unwrap();
        let output_path = self.output_path;

        let is_profiling = self.is_profiling.clone();

        thread::spawn(move || {
            let mut batch = Batch::new(output_path, start_time);

            loop {
                if is_profiling.load(Ordering::Relaxed) {
                    let result: ProfileObject = rx.recv().unwrap();

                    batch.add(result);

                    if batch.can_flush() {
                        batch.flush();
                    }
                }
            }
        });
    }

    pub fn end_session(&mut self) {
        self.is_profiling
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);

        if let Ok(lines) = Self::read_lines(self.output_path) {
            let mut sorted_vec = lines
                .filter(|x| {
                    if let Ok(x) = x {
                        if x == "" {
                            return false;
                        }
                    }
                    true
                })
                .map(|x| {
                    let line = x.unwrap();
                    let results = line.split(",").collect::<Vec<&str>>();
                    SerializedProfileObject::new(
                        results[0].to_owned(),
                        results[1].parse().unwrap(),
                        results[2].parse().unwrap(),
                    )
                })
                .collect::<Vec<SerializedProfileObject>>();

            sorted_vec.sort_by(|a, b| a.ts.partial_cmp(&b.ts).unwrap());

            self.remove_file();

            Batch::write_to_file(self.output_path, "{\"traceEvents\":");
            Batch::write_to_file(
                self.output_path,
                &serde_json::ser::to_string(&sorted_vec).unwrap(),
            );
            Batch::write_to_file(self.output_path, "}");
        }
    }

    fn remove_file(&self) {
        fs::remove_file(self.output_path);
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedProfileObject {
    cat: &'static str,
    dur: u128,
    name: String,
    ph: &'static str,
    pid: u32,
    tid: u32,
    ts: u128,
}

impl SerializedProfileObject {
    pub fn new(name: String, duration: u128, start: u128) -> SerializedProfileObject {
        SerializedProfileObject {
            cat: "function",
            dur: duration,
            name,
            ph: "X",
            pid: 0,
            tid: 1,
            ts: start,
        }
    }
}

#[derive(Clone)]
pub struct ProfileObject {
    pub start_time: Instant,
    pub duration: Duration,
    pub profile_fn: String,
}

impl ProfileObject {
    pub fn new(profile_fn: String) -> ProfileObject {
        ProfileObject {
            start_time: get_current_time_ns(),
            duration: Duration::default(),
            profile_fn,
        }
    }

    pub fn end_profile(self) {
        let mut lock = PROFILER.lock().unwrap();
        lock.end_profile(self);
        drop(lock)
    }
}

fn get_current_time_ns() -> Instant {
    Instant::now()
}

pub fn start_profiler() {
    let mut profiler = PROFILER.lock().unwrap();
    profiler.start_session();
    drop(profiler);
}

pub fn stop_profiler() {
    let mut profiler = PROFILER.lock().unwrap();
    profiler.end_session();
    drop(profiler);
}
