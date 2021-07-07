#[cfg(feature = "profile")]
mod profile_enabled;

#[cfg(not(feature = "profile"))]
mod profile_disabled;

#[cfg(not(feature = "profile"))]
pub use profile_disabled::{PROFILER, start_profiler, stop_profiler};

#[cfg(feature = "profile")]
pub use profile_enabled::{PROFILER, start_profiler, stop_profiler};
