use std::time::{SystemTime, Duration, Instant};

fn main() {
    let start = Instant::now();

    loop {
        let end = Instant::now();
        let elapsed = end.duration_since(start);
        println!("{:?} {:?} {:?}", start, end, elapsed);
    }
}
