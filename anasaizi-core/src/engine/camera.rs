use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

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
    camera_up: nalgebra::Vector3<f32>,
    camera_front: nalgebra::Vector3<f32>,
    position: nalgebra::Point3<f32>,

    projection_matrix: nalgebra::Matrix4<f32>,
    view_matrix: nalgebra::Matrix4<f32>,

    speed: f32,
    rotation_speed: f32,

    yaw: f64,
    pitch: f64,
    roll: f64,

    field_of_view: f32,
    aspect_ratio: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32, field_of_view: f32, znear: f32, zfar: f32) -> Self {
        let position: nalgebra::Point3<f32> = nalgebra::Point3::<f32>::new(0.0, 0.0, 3.0);
        let camera_front: nalgebra::Vector3<f32> = nalgebra::Vector3::<f32>::new(0.0, 0.0, -1.0);
        let camera_up: nalgebra::Vector3<f32> = nalgebra::Vector3::<f32>::new(0.0, 1.0, 0.0);

        let camera_target: nalgebra::Point3<f32> = Self::add_vector(&position, &camera_front);

        let view_matrix = nalgebra::Matrix4::look_at_rh(&position, &camera_target, &camera_up);

        let mut projection_matrix: nalgebra::Matrix4<f32> =
            nalgebra::Perspective3::new(aspect_ratio, field_of_view, znear, zfar).to_homogeneous();
        projection_matrix[(1, 1)] = projection_matrix[(1, 1)] * -1.0;

        Camera {
            view_matrix,
            projection_matrix,

            position: position.into(),
            camera_up,
            camera_front,

            speed: 0.05,
            rotation_speed: 0.1,

            aspect_ratio,

            yaw: -90.0,
            pitch: 0.0,
            roll: 0.0,
            field_of_view,

            znear,
            zfar,
        }
    }

    pub fn projection(&self) -> nalgebra::Matrix4<f32> {
        self.projection_matrix
    }

    pub fn view(&self) -> nalgebra::Matrix4<f32> {
        self.view_matrix
    }

    pub fn process_key(&mut self, direction: CameraMovement, delta_time: f32) {
        let velocity = self.speed;

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

    pub fn process_mouse_scroll(&mut self, mut yoffset: f32) {
        self.field_of_view -= yoffset;

        if self.field_of_view < 1.0 {
            self.field_of_view = 1.0;
        }
        if self.field_of_view > 45.0 {
            self.field_of_view = 45.0;
        }

        self.reload_perspective();
    }

    pub fn process_mouse(&mut self, _delta_time: f32, mut xoffset: f64, mut yoffset: f64) {
        let sensitivity = 0.1;
        xoffset *= sensitivity;
        yoffset *= sensitivity;

        self.add_yaw(xoffset);
        self.add_pitch(yoffset);

        let mut direction = nalgebra::Vector3::<f32>::default();
        direction[0] =
            Self::get_radians(self.yaw as f32).cos() * Self::get_radians(self.pitch as f32).cos();
        direction[1] = Self::get_radians(self.pitch as f32).sin();
        direction[2] =
            Self::get_radians(self.yaw as f32).sin() * Self::get_radians(self.pitch as f32).cos();

        self.camera_front = direction.normalize().into();

        self.reload_matrix();
    }

    fn reload_matrix(&mut self) {
        let camera_target: nalgebra::Point3<f32> =
            Self::add_vector(&mut self.position, &self.camera_front);

        let view_matrix =
            nalgebra::Matrix4::look_at_rh(&self.position, &camera_target, &self.camera_up);

        self.view_matrix = view_matrix;
    }

    fn reload_perspective(&mut self) {
        let mut projection_matrix: nalgebra::Matrix4<f32> = nalgebra::Perspective3::new(
            self.aspect_ratio,
            self.field_of_view,
            self.znear,
            self.zfar,
        )
        .to_homogeneous();
        projection_matrix[(1, 1)] = projection_matrix[(1, 1)] * -1.0;

        self.projection_matrix = projection_matrix;
    }

    fn add_vector(
        position: &nalgebra::Point3<f32>,
        direction: &nalgebra::Vector3<f32>,
    ) -> nalgebra::Point3<f32> {
        nalgebra::Point3::<f32>::new(
            position[0] + direction[0],
            position[1] + direction[1],
            position[2] + direction[2],
        )
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
