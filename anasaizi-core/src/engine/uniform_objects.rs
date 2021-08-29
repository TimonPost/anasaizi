use std::mem::size_of;
use nalgebra::Vector3;
use std::any::Any;

/// Template for an uniform buffer object.
pub trait UniformObjectTemplate : UniformObjectClone  {
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
        T: 'static + UniformObjectTemplate + Clone+ Default,
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
    pub position: Vector3<f32>,

    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,

    pub view_pos: Vector3<f32>,
    pub light_color: Vector3<f32>,
}

impl Default for LightUniformObject {
    fn default() -> Self {
        LightUniformObject {
            position: Vector3::default(),
            light_color: Vector3::new(1.0,1.0,1.0),
            view_pos: Vector3::default(),
            ambient : Vector3::new(0.1,0.1,0.1),
            specular: Vector3::new(1.0,1.0,1.0),
            diffuse: Vector3::new(1.0,1.0,1.0),

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

/// Uniform buffer object.
#[derive(Clone, Copy)]
pub struct MatrixUniformObject {
    pub view_matrix: nalgebra::Matrix4<f32>,
    pub projection_matrix: nalgebra::Matrix4<f32>,
}

impl UniformObjectTemplate for MatrixUniformObject {
    fn size(&self) -> usize {
        size_of::<MatrixUniformObject>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for MatrixUniformObject {
    fn default() -> Self {
        let mut identity = nalgebra::Matrix4::default();
        identity.fill_with_identity();

        MatrixUniformObject {
            view_matrix: identity,
            projection_matrix: identity,
        }
    }
}

/// Uniform buffer object.
#[derive(Clone, Copy)]
pub struct MaterialUniformObject{
    pub shininess:f32,
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
}

impl MaterialUniformObject {
    pub fn new(shininess: f32, ambient: Vector3<f32>, diffuse: Vector3<f32>, specular: Vector3<f32>) -> MaterialUniformObject {
        MaterialUniformObject {
            shininess,
            ambient,
            diffuse,
            specular
        }
    }
}

impl UniformObjectTemplate for MaterialUniformObject {
    fn size(&self) -> usize {
        size_of::<MaterialUniformObject>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for MaterialUniformObject {
    fn default() -> Self {
        MaterialUniformObject::new(32.0, Vector3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,0.0))
    }
}