#[macro_use]
use lazy_static::lazy_static;

#[macro_export]
macro_rules! profile_fn {
    ($profile:expr, $to_profile:expr) => {{
        $to_profile
    }};
}

lazy_static! {
    /// This is an example for using doc comment attributes
    pub static ref PROFILER: i32 = 0;
}

pub fn start_profiler() {}

pub fn stop_profiler() {}
