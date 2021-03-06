mod vertex;

pub use vertex::{GltsVertex, PosOnlyVertex, Vertex};

pub type Vector3 = nalgebra::Vector3<f32>;
pub type Vector4 = nalgebra::Vector4<f32>;
pub type Quaternion = nalgebra::Quaternion<f32>;
pub type Matrix4 = nalgebra::Matrix4<f32>;
pub type Vector2 = nalgebra::Vector2<f32>;
pub type Point3 = nalgebra::Point3<f32>;
