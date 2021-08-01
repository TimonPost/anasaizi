use crate::vulkan::{
    structures::ValidationInfo, Application, Extensions, LogicalDevice, ValidationLayers,
};
use ash::{version::EntryV1_0, vk};
use std::{ffi::CString, fmt, ops::Deref};

pub struct Instance {
    entry: ash::Entry,
    instance: ash::Instance,

    validation_layers: Option<ValidationLayers>,
}

impl Instance {
    pub fn new(
        validation: ValidationInfo,
        extensions: Extensions,
        application: &Application,
    ) -> Instance {
        // Create entry
        let entry = unsafe { ash::Entry::new().unwrap() };

        // Check if validation layers are enabled
        let mut validation_layers = None;

        if validation.is_enable {
            let layers = ValidationLayers::new(&entry, validation.to_vec_owned());

            if !layers.has_required_layers() {
                panic!("Validation layers requested, but not available!");
            }

            validation_layers = Some(layers)
        }

        // Get enabled layers
        let enabled_layers = validation.to_vec_ptr();

        let enabled_layers_ptr = enabled_layers
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect::<Vec<*const i8>>();

        // Get extensions
        let extensions_raw = extensions.as_cstrings();
        let extensions_ptr = extensions_raw
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<*const i8>>();

        // Build instance
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&CString::new(application.app_name).expect("No valid cstring"))
            .application_version(application.app_version.encode())
            .engine_name(&CString::new(application.engine_name).expect("No valid cstring"))
            .engine_version(application.engine_version.encode())
            .api_version(application.api_version.encode())
            .build();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(if validation.is_enable {
                &enabled_layers_ptr
            } else {
                &[]
            })
            .enabled_extension_names(&extensions_ptr);

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance!")
        };

        Instance {
            entry,
            validation_layers,
            instance,
        }
    }

    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }
}

impl Deref for Instance {
    type Target = ash::Instance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if let Some(validation_layer) = &self.validation_layers {
            write!(f, "{:?}", validation_layer)?;
        }

        Ok(())
    }
}
