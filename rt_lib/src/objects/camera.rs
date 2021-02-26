use glam::*;

pub struct Camera {
    pub eye: Vec3,
    pub look_at: Vec3,

    pub fov: f32, //In degrees
}

impl Camera {
    pub fn default() -> Self {
        Self {
            eye: vec3(0.0,0.0,-5.0),
            look_at: vec3(0.0,0.0,0.0),

            fov: 60.0,
        }
    }

    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov / 180.0 * std::f32::consts::PI, aspect_ratio, 0.02, 1024.0)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.look_at, vec3(0.0,1.0,0.0))
    }
}
