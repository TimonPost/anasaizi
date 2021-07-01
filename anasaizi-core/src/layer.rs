use ash::version::EntryV1_0;
use std::fmt;
use crate::vk_to_string;

pub struct ValidationLayerProperties {
    pub name: String,
    pub description: String,
    pub specs_version: u32,
    pub implementation_version: u32,
}

pub struct ValidationLayers {
    supported_layers: Vec<ValidationLayerProperties>,
    required_layers: Vec<String>,
}

impl ValidationLayers {
    pub fn new(entry: &ash::Entry, required_layers: Vec<String>) -> ValidationLayers {
        ValidationLayers {
            supported_layers: Self::initialize_validation_layers(entry),
            required_layers,
        }
    }

    pub fn initialize_validation_layers(entry: &ash::Entry) -> Vec<ValidationLayerProperties> {
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate Instance Layers Properties!");

        let mut supported_layers = vec![];

        for layer in layer_properties.iter() {
            let layer_name = vk_to_string(&layer.layer_name).unwrap();
            let description = vk_to_string(&layer.description).unwrap();
            let specs_version = layer.spec_version;
            let implementation_version = layer.implementation_version;

            supported_layers.push(ValidationLayerProperties {
                name: layer_name,
                description,
                specs_version,
                implementation_version,
            });
        }

        supported_layers
    }

    pub fn has_required_layers(&self) -> bool {
        for required_layer in self.required_layers.iter() {
            let contains_layer = self
                .supported_layers
                .iter()
                .any(|l| l.name == *required_layer);

            if !contains_layer {
                return false;
            }
        }

        true
    }
}

impl fmt::Debug for ValidationLayers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Required Layers:\n")?;

        for required_layer in &self.required_layers {
            write!(f, "\t - {}\n", required_layer)?;
        }

        write!(f, "\nSupported Layers:\n")?;

        for supported_layer in &self.supported_layers {
            write!(
                f,
                "\t - name: {}; description: {}; spec-version: {}; implementation-version: {}\n",
                supported_layer.name,
                supported_layer.description,
                supported_layer.specs_version,
                supported_layer.implementation_version
            )?;
        }

        Ok(())
    }
}
