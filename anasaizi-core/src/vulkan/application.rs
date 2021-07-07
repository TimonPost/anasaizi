use crate::vulkan::Version;
use std::fmt;

/// A Vulkan application instance.
pub struct Application {
    pub app_name: &'static str,
    pub engine_name: &'static str,
    pub app_version: Version,
    pub engine_version: Version,
    pub api_version: Version,
    pub window_width: u32,
    pub window_height: u32,
}

impl Application {
    /// Creates a new vulkan application instance.
    pub fn new(
        app_name: &'static str,
        engine_name: &'static str,
        app_version: Version,
        engine_version: Version,
        api_version: Version,
        window_width: u32,
        window_height: u32,
    ) -> Application {
        Application {
            app_version,
            engine_version,
            api_version,
            window_width,
            window_height,
            app_name,
            engine_name,
        }
    }
}

impl fmt::Debug for Application {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "\n Application Info:\n")?;
        write!(f, "\t - App: {:?} {:?}\n", self.app_name, self.app_version)?;
        write!(
            f,
            "\t - Engine: {:?} {:?}\n",
            self.engine_name, self.engine_version
        )?;
        write!(f, "\t - Vulkan API: {:?}\n", self.api_version)?;
        write!(
            f,
            "\t - Width/Height: {}/{}\n",
            self.window_width, self.window_height
        )?;

        Ok(())
    }
}
