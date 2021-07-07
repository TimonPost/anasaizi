#[cfg(feature = "profile")]
mod profile_enabled;

#[cfg(not(feature = "profile"))]
mod profile_disabled;

#[cfg(not(feature = "profile"))]
pub use profile_disabled::{start_profiler, stop_profiler, PROFILER};

#[cfg(feature = "profile")]
pub use profile_enabled::{start_profiler, stop_profiler, PROFILER};
