use bytemuck::{Pod, Zeroable};
use cgmath::{
    EuclideanSpace, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3, Vector2,
    Vector3,
};

const DELTA_POSITION: f32 = 0.05;
const DELTA_ANGLE: f32 = std::f32::consts::PI / 100.0;
const ELEVATION_MARGIN_RATIO: f32 = 0.001;
const MAX_ELEVATION_ANGLE: Rad<f32> =
    Rad(std::f32::consts::FRAC_PI_2 * (1.0 - ELEVATION_MARGIN_RATIO));
const MIN_ELEVATION_ANGLE: Rad<f32> =
    Rad(-std::f32::consts::FRAC_PI_2 * (1.0 - ELEVATION_MARGIN_RATIO));

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug)]
pub struct Camera {
    position: Point3<f32>,
    dir: Vector3<f32>,
    up: Vector3<f32>,
    fov: f32,
    near: f32,
    far: f32,

    should_move_right: bool,
    should_move_left: bool,
    should_move_forward: bool,
    should_move_backward: bool,
    should_move_up: bool,
    should_move_down: bool,
    should_turn_left: bool,
    should_turn_right: bool,
    should_look_up: bool,
    should_look_down: bool,
}

impl Default for Camera {
    fn default() -> Self {
        let position = Point3::new(0.0, 0.0, 5.0);
        let dir = -position.to_vec();
        let up = Vector3::unit_y();
        let fov = 45f32;
        let near = 0.1;
        let far = 1000.0;

        Camera {
            position,
            dir,
            up,
            fov,
            near,
            far,
            should_move_right: false,
            should_move_left: false,
            should_move_forward: false,
            should_move_backward: false,
            should_move_up: false,
            should_move_down: false,
            should_turn_left: false,
            should_turn_right: false,
            should_look_up: false,
            should_look_down: false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RawCamera {
    pub vp_matrix: [f32; 16],
    pub camera_pos: [f32; 3],
}

impl Camera {
    pub fn create_raw_camera(&self, aspect_ratio: f32) -> RawCamera {
        let projection_matrix =
            cgmath::perspective(cgmath::Deg(self.fov), aspect_ratio, self.near, self.far);
        let view_matrix = Matrix4::look_to_rh(self.position, self.dir, self.up);
        let vp_matrix = OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix;
        let vp_matrix = *vp_matrix.as_ref();
        let camera_pos = self.position.into();
        RawCamera {
            vp_matrix,
            camera_pos,
        }
    }

    pub fn move_right(&mut self, should_move: bool) {
        self.should_move_right = should_move;
    }

    pub fn move_left(&mut self, should_move: bool) {
        self.should_move_left = should_move;
    }

    pub fn move_forward(&mut self, should_move: bool) {
        self.should_move_forward = should_move;
    }

    pub fn move_backward(&mut self, should_move: bool) {
        self.should_move_backward = should_move;
    }

    pub fn move_up(&mut self, should_move: bool) {
        self.should_move_up = should_move;
    }

    pub fn move_down(&mut self, should_move: bool) {
        self.should_move_down = should_move;
    }

    pub fn turn_left(&mut self, should_turn: bool) {
        self.should_turn_left = should_turn;
    }

    pub fn turn_right(&mut self, should_turn: bool) {
        self.should_turn_right = should_turn;
    }

    pub fn look_up(&mut self, should_look: bool) {
        self.should_look_up = should_look;
    }

    pub fn look_down(&mut self, should_look: bool) {
        self.should_look_down = should_look;
    }

    pub fn update(&mut self) {
        let forward_dir = self.dir_xz_projection();
        let up_dir = Vector3::unit_y();
        let right_dir = forward_dir.cross(up_dir);

        let horizontal_angle =
            DELTA_ANGLE * (self.should_turn_left as i8 - self.should_turn_right as i8) as f32;
        let horizontal_rotation = Quaternion::from_axis_angle(up_dir, Rad(horizontal_angle));
        self.dir = horizontal_rotation.rotate_vector(self.dir);

        let elevation_angle = self.elevation_angle();
        let mut vertical_angle =
            Rad(DELTA_ANGLE * (self.should_look_up as i8 - self.should_look_down as i8) as f32);
        if elevation_angle + vertical_angle >= MAX_ELEVATION_ANGLE {
            vertical_angle = MAX_ELEVATION_ANGLE - elevation_angle;
        } else if elevation_angle + vertical_angle <= MIN_ELEVATION_ANGLE {
            vertical_angle = MIN_ELEVATION_ANGLE - elevation_angle;
        }
        let vertical_rotation = Quaternion::from_axis_angle(right_dir, vertical_angle);
        self.dir = vertical_rotation.rotate_vector(self.dir);

        let delta_forward =
            (self.should_move_forward as i8 - self.should_move_backward as i8) as f32 * forward_dir;
        let delta_up = (self.should_move_up as i8 - self.should_move_down as i8) as f32 * up_dir;
        let delta_right =
            (self.should_move_right as i8 - self.should_move_left as i8) as f32 * right_dir;
        self.position += DELTA_POSITION * (delta_forward + delta_up + delta_right)
    }

    fn elevation_angle(&self) -> Rad<f32> {
        let r = Vector2::new(self.dir.x, self.dir.z).magnitude();
        Rad(self.dir.y.atan2(r))
    }

    fn dir_xz_projection(&self) -> Vector3<f32> {
        Vector3::new(self.dir.x, 0.0, self.dir.z).normalize()
    }
}
