use crate::math::{Matrix4, Vector3};
use std::ops::RangeInclusive;

/// The transform of an object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub scale_transform: Matrix4,
    pub rotate_transform: Matrix4,
    pub translate_transform: Matrix4,

    pub unit_scale: f32,
    rotation_factor: Vector3,
    pub parent: Matrix4,
}

impl Transform {
    pub fn new(unit_scale: f32) -> Transform {
        let mut identity = Matrix4::default();
        identity.fill_with_identity();

        Transform {
            scale_transform: identity,
            rotate_transform: identity,
            translate_transform: identity,
            rotation_factor: Vector3::default(),
            unit_scale,
            parent: identity,
        }
    }

    pub fn unit_scale(&self) -> RangeInclusive<f32> {
        0.0..=self.unit_scale
    }

    pub fn with_parent_transform(mut self, transform: Matrix4) -> Transform {
        self.parent = transform;
        self
    }

    pub fn with_const_scale(self, factor: f32) -> Transform {
        self.with_scale(Vector3::new(factor, factor, factor));
        self
    }

    pub fn with_scale(mut self, factor: Vector3) -> Transform {
        self.scale(factor);
        self
    }

    pub fn with_translate(mut self, translate: Vector3) -> Transform {
        self.translate(translate);
        self
    }

    pub fn with_rotation(mut self, rotation: Vector3) -> Transform {
        self.rotate(rotation);
        self
    }

    pub fn rotate(&mut self, rotate: Vector3) {
        self.rotation_factor = rotate;
        self.rotate_transform = Matrix4::new_rotation(rotate);
    }
    pub fn translate(&mut self, translate: Vector3) {
        let translate_matrix = Matrix4::new(
            1.0,
            0.0,
            0.0,
            translate[0],
            0.0,
            1.0,
            0.0,
            translate[1],
            0.0,
            0.0,
            1.0,
            translate[2],
            0.0,
            0.0,
            0.0,
            1.0,
        );

        self.translate_transform = translate_matrix;
    }

    pub fn scale(&mut self, factor: Vector3) {
        let scale_matrix = Matrix4::new(
            factor[0], 0.0, 0.0, 0.0, 0.0, factor[1], 0.0, 0.0, 0.0, 0.0, factor[2], 0.0, 0.0, 0.0,
            0.0, 1.0,
        );

        self.scale_transform = scale_matrix;
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_transform[(0, 0)]
    }

    pub fn rotation_factor(&self) -> Vector3 {
        self.rotation_factor
    }

    pub fn translate_factor(&self) -> Vector3 {
        let x = self.translate_transform[(0, 3)];
        let y = self.translate_transform[(1, 3)];
        let z = self.translate_transform[(2, 3)];

        Vector3::new(x, y, z)
    }

    pub fn model_transform(&self) -> nalgebra::Matrix4<f32> {
        return self.parent
            * self.rotate_transform
            * self.scale_transform
            * self.translate_transform;
    }
}
