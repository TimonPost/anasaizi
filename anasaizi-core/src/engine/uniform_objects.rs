use nalgebra::Vector4;
use std::{any::Any, mem::size_of};

/// Template for an uniform buffer object.
pub trait UniformObjectTemplate: UniformObjectClone {
    /// Returns the size of this buffer object.
    fn size(&self) -> usize;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait UniformObjectClone {
    fn clone_box(&self) -> Box<dyn UniformObjectTemplate>;
}

impl<T> UniformObjectClone for T
where
    T: 'static + UniformObjectTemplate + Clone + Default,
{
    fn clone_box(&self) -> Box<dyn UniformObjectTemplate> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn UniformObjectTemplate> {
    fn clone(&self) -> Box<dyn UniformObjectTemplate> {
        self.clone_box()
    }
}

#[derive(Clone, Copy)]
pub struct LightUniformObject {
    pub position: Vector4<f32>,
    pub view_pos: Vector4<f32>,
    pub light_color: Vector4<f32>,
}

impl Default for LightUniformObject {
    fn default() -> Self {
        LightUniformObject {
            position: Vector4::default(),
            light_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            view_pos: Vector4::default(),
        }
    }
}

impl UniformObjectTemplate for LightUniformObject {
    fn size(&self) -> usize {
        size_of::<LightUniformObject>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Clone, Copy)]
pub struct GLTFLightUniformObject {
    pub position: Vector4<f32>,
    pub view_pos: Vector4<f32>,
    pub light_color: Vector4<f32>,
    pub ambient_color: Vector4<f32>,
    pub light_direction: Vector4<f32>,
    pub ambient_light_intensity: f32,
}

impl Default for GLTFLightUniformObject {
    fn default() -> Self {
        GLTFLightUniformObject {
            position: Vector4::default(),
            light_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            ambient_color: Default::default(),
            ambient_light_intensity: Default::default(),
            view_pos: Vector4::default(),
            light_direction: Default::default(),
        }
    }
}

impl UniformObjectTemplate for GLTFLightUniformObject {
    fn size(&self) -> usize {
        size_of::<GLTFLightUniformObject>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Uniform buffer object.
#[derive(Clone, Copy)]
pub struct ViewProjectionMatrixUniformObject {
    pub view_matrix: nalgebra::Matrix4<f32>,
    pub projection_matrix: nalgebra::Matrix4<f32>,
}

impl UniformObjectTemplate for ViewProjectionMatrixUniformObject {
    fn size(&self) -> usize {
        size_of::<ViewProjectionMatrixUniformObject>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for ViewProjectionMatrixUniformObject {
    fn default() -> Self {
        let mut identity = nalgebra::Matrix4::default();
        identity.fill_with_identity();

        ViewProjectionMatrixUniformObject {
            view_matrix: identity,
            projection_matrix: identity,
        }
    }
}
