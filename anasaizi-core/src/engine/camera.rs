use nalgebra::{Vector3, Vector4};

/// Defines in which direction a camera should move.
pub enum CameraMovement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
}

/// Perspective camera.
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

    field_of_view: f32,
    aspect_ratio: f32,
    znear: f32,
    zfar: f32,

    is_dirty: bool,
}

impl Camera {
    /// Creates a new camera with the given aspect ratio, field of view, and near/far planes.
    pub fn new(aspect_ratio: f32, field_of_view: f32, znear: f32, zfar: f32) -> Self {
        let position: nalgebra::Point3<f32> = nalgebra::Point3::<f32>::new(0.0, 1.0, 3.0);
        let camera_front: nalgebra::Vector3<f32> = nalgebra::Vector3::<f32>::new(0.0, 0.0, -1.0);
        let camera_up: nalgebra::Vector3<f32> = nalgebra::Vector3::<f32>::new(0.0, 1.0, 0.0);

        let camera_target: nalgebra::Point3<f32> = Self::add_vector(&position, &camera_front);

        let view_matrix = nalgebra::Matrix4::look_at_lh(&position, &camera_target, &camera_up);

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
            field_of_view,

            znear,
            zfar,

            is_dirty: true,
        }
    }

    pub fn update_screen_resize(&mut self, aspect_ratio: f32, fov: f32) {
        self.aspect_ratio = aspect_ratio;
        self.field_of_view = fov;
        self.mark_dirty();
    }

    pub fn position(&self) -> Vector3<f32> {
        Vector3::new(self.position.x, self.position.y, self.position.z)
    }

    /// Returns the projection matrix.
    pub fn projection(&self) -> nalgebra::Matrix4<f32> {
        self.projection_matrix
    }

    /// Returns the view matrix.
    pub fn view(&self) -> nalgebra::Matrix4<f32> {
        self.view_matrix
    }

    /// Process movement slowly updating the camera position with delta time.
    pub fn process_movement(&mut self, direction: CameraMovement, _delta_time: f32) {
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

        self.mark_dirty();
    }

    /// Returns if the camera needs to be recalculated.
    pub fn needs_recalculation(&self) -> bool {
        self.is_dirty
    }

    /// Reloads the camera matrices, vectors, and angle properties.
    pub fn reload(&mut self) {
        self.reload_perspective();
        self.reload_matrix();
        self.is_dirty = false;
    }

    pub fn screen_to_world(&self, position: Vector4<f32>) -> Vector4<f32> {
        let inverse_projection = self.projection().try_inverse().unwrap();
        let inverse_view = self.view().try_inverse().unwrap();

        let world_position = inverse_projection * inverse_view * position;

        world_position
    }

    /// Processes mouse scroll.
    ///
    /// This updates the camera field of view and therefore simulates a zoom action.
    pub fn process_mouse_scroll(&mut self, yoffset: f32) {
        self.field_of_view -= yoffset;

        if self.field_of_view < 1.0 {
            self.field_of_view = 1.0;
        }
        if self.field_of_view > 45.0 {
            self.field_of_view = 45.0;
        }

        self.mark_dirty();
    }

    /// Processes mouse movement.
    /// This function will update the direction based on mouse, yaw, pitch values.
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

        self.mark_dirty();
    }

    /// Adds a pitch value to the current pitch.
    /// Pitch has can not be more then 89 or less then -89 degrees.
    pub fn add_pitch(&mut self, pitch: f64) {
        let new_pitch = self.pitch + pitch;

        if new_pitch > 89.0 {
            self.pitch = 89.0;
        } else if new_pitch < -89.0 {
            self.pitch = -89.0;
        } else {
            self.pitch = new_pitch;
        }
    }

    /// Adds the given amount of yaw to the camera yaw value.
    pub fn add_yaw(&mut self, yaw: f64) {
        self.yaw += yaw;
    }

    fn mark_dirty(&mut self) {
        self.is_dirty = true;
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

    fn get_radians(degrees: f32) -> f32 {
        degrees * (std::f64::consts::PI as f32 / 180.0)
    }
}
