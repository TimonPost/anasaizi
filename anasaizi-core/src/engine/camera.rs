use crate::reexports::nalgebra::{ArrayStorage, Const, Matrix};
use nalgebra::{RealField, Vector3};

pub enum CameraMovement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
}

const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 40.5;
const SENSITIVITY: f32 = 0.4;
const ZOOM: f32 = 45.0;

pub struct Camera {
    up_vector: nalgebra::Vector3<f32>,
    direction: nalgebra::Vector3<f32>,
    position: nalgebra::Vector3<f32>,

    projection_matrix: nalgebra::Matrix4<f32>,
    view_matrix: nalgebra::Matrix4<f32>,

    speed: f32,
    rotation_speed: f32,
    zoom_level: f32,

    yaw: f64,
    pitch: f64,
    roll: f64,

    field_of_view: f32,
    aspect_ratio: f32,

    camera_front: nalgebra::Vector3<f32>,
    camera_up: nalgebra::Vector3<f32>,
    camera_right: nalgebra::Vector3<f32>,
}

impl Camera {
    pub fn new(aspect_ratio: f32, field_of_view: f32, znear: f32, zfar: f32) -> Self {
        let up_vector: nalgebra::Vector3<f32> = nalgebra::Vector3::<f32>::new(0.0, 1.0, 0.0);
        let target: nalgebra::Vector3<f32> = nalgebra::Vector3::<f32>::new(0.0, 0.0, 0.0);
        let position: nalgebra::Vector3<f32> = nalgebra::Vector3::<f32>::new(0.0, 2.0, 2.0);

        let direction = (position - target).normalize();
        let camera_right: nalgebra::Vector3<f32> = up_vector.cross(&direction);
        let camera_up: nalgebra::Vector3<f32> = direction.cross(&camera_right);
        let camera_front: nalgebra::Vector3<f32> = nalgebra::Vector3::new(0.0, 0.0, -1.0);

        let camera_target: nalgebra::Vector3<f32> = position + camera_front;

        let view_matrix = nalgebra::Matrix4::look_at_rh(
            &nalgebra::Point3::new(position[0], position[1], position[2]),
            &nalgebra::Point3::new(camera_target[0], camera_target[1], camera_target[2]),
            &up_vector,
        );

        let projection_matrix =
            nalgebra::Perspective3::new(aspect_ratio, field_of_view, znear, zfar);

        let mut projection_matrix = *projection_matrix.as_matrix();
        projection_matrix[(1, 1)] = projection_matrix[(1, 1)] * -1.0;

        Camera {
            up_vector,

            view_matrix,
            projection_matrix,

            position: position.into(),
            direction,

            camera_up,
            camera_front,
            camera_right,

            speed: 0.05,
            rotation_speed: 0.1,
            zoom_level: 1.0,

            aspect_ratio,

            yaw: -90.0,
            pitch: 0.0,
            roll: 0.0,
            field_of_view,
        }
    }

    pub fn projection(&self) -> nalgebra::Matrix4<f32> {
        self.projection_matrix
    }

    pub fn view(&self) -> nalgebra::Matrix4<f32> {
        self.view_matrix
    }

    pub fn process_key(&mut self, direction: CameraMovement, delta_time: f32) {
        let velocity = self.speed * delta_time;

        match direction {
            CameraMovement::FORWARD => {
                self.position += self.camera_front * velocity;
            }
            CameraMovement::BACKWARD => {
                self.position -= self.camera_front * velocity;
            }
            CameraMovement::LEFT => {
                self.position -= self.camera_front.cross(&self.camera_up).normalize() * velocity;
            }
            CameraMovement::RIGHT => {
                self.position += self.camera_front.cross(&self.camera_up).normalize() * velocity;
            }
        }

        self.reload_matrix();
    }

    pub fn process_mouse(&mut self, delta_time: f32, mut xoffset: f64, mut yoffset: f64) {
        let sensitivity = 0.1;
        xoffset *= sensitivity;
        yoffset *= sensitivity;

        self.add_yaw(xoffset);
        self.add_pitch(yoffset);

        self.reload_matrix();
    }

    pub fn update_orientation(&mut self) {
        let mut direction = nalgebra::Vector3::<f32>::default();
        direction[0] =
            Self::get_radians(self.yaw as f32).cos() * Self::get_radians(self.pitch as f32).cos();
        direction[1] = Self::get_radians(self.pitch as f32).sin();
        direction[2] =
            Self::get_radians(self.yaw as f32).sin() * Self::get_radians(self.pitch as f32).sin();
        self.camera_front = direction.normalize();
    }

    fn reload_matrix(&mut self) {
        self.update_orientation();

        let camera_target: nalgebra::Vector3<f32> = self.position + self.camera_front;

        let view_matrix = nalgebra::Matrix4::look_at_rh(
            &nalgebra::Point3::new(self.position[0], self.position[1], self.position[2]),
            &nalgebra::Point3::new(camera_target[0], camera_target[1], camera_target[2]),
            &self.up_vector,
        );

        self.view_matrix = view_matrix;
    }

    pub(crate) fn add_pitch(&mut self, pitch: f64) {
        let new_pitch = self.pitch + pitch;

        if new_pitch > 89.0 {
            self.pitch = 89.0;
        } else if new_pitch < -89.0 {
            self.pitch = -89.0;
        } else {
            self.pitch = new_pitch;
        }
    }

    pub(crate) fn add_yaw(&mut self, yaw: f64) {
        self.yaw += yaw;
    }

    fn get_radians(degrees: f32) -> f32 {
        degrees * (std::f64::consts::PI as f32 / 180.0)
    }
}
