use glam::*;

use glux::gl_types::Texture;

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Ray {
    pos: glux::gl_types::f32_f32_f32_f32,
    dir: glux::gl_types::f32_f32_f32_f32,
}

impl Ray {
    pub fn default() -> Self {
        Self {
            pos: glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,-5.0, 0.0),
            dir: glux::gl_types::f32_f32_f32_f32::new(0.0,0.0, 1.0, 0.0),
        }
    }
}

pub struct Camera {
    pub eye: Vec3,
    pub look_at: Vec3,

    pub fov: f32, //In degrees

    pub resolution: (usize, usize), //Output resolution
    pub render_buffer: Texture,     //Output buffer
}

impl Camera {
    pub fn new(resolution: (usize, usize)) -> Self {
        let texture = Texture::from_ptr((resolution.0 as i32, resolution.1 as i32), std::ptr::null(), gl::RGBA32F as i32, gl::RGBA);
        unsafe {
            gl::BindImageTexture(0, texture.id, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F);
        }
        trace!("Render texture constructed!");

        Self {
            eye: vec3(0.0,0.0,-5.0),
            look_at: vec3(0.0,0.0,0.0),

            fov: 60.0,

            resolution: resolution,
            render_buffer: texture,
        }
    }

    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov / 180.0 * std::f32::consts::PI, aspect_ratio, 0.02, 1024.0)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.look_at, vec3(0.0,1.0,0.0))
    }

    //TODO: Move ray generation to a shader
    pub fn generate_rays(&self, ssbo: &glux::gl_types::ShaderStorageBuffer) {
        let inv_proj_view = (self.get_projection_matrix(self.resolution.0 as f32 / self.resolution.1 as f32) * self.get_view_matrix()).inverse();

        let mut data = vec![Ray::default(); self.resolution.0 * self.resolution.1];
        trace!("Empty buffer constructed!");

        for x in 0..self.resolution.0 {
            for y in 0..self.resolution.1 {
                let uv = vec2(x as f32 / self.resolution.0 as f32, y as f32 / self.resolution.1 as f32);
                let ray = ray_from_projview(uv, inv_proj_view);
                let data_index = x + y * self.resolution.0;
                data[data_index] = ray;
            }
        }

        trace!("Rays generated!");

        ssbo.bind();
        ssbo.data(&data[..], gl::DYNAMIC_COPY);
        ssbo.unbind();

        trace!("Ray data uploaded!");
    }
}

fn ray_from_projview(uv: Vec2, inv_proj_view: Mat4) -> Ray {
    let pos = uv * 2.0 - vec2(1.0,1.0);
    let near = 0.02;
    let far = 1024.0;
    let origin = (inv_proj_view * vec4(pos.x, pos.y, -1.0, 1.0) * near).xyz();
    let dir = {
        let tmp = pos * (far-near);
        (inv_proj_view * vec4(tmp.x, tmp.y, far + near, far - near)).xyz().normalize()
    };
    Ray {
        pos: glux::gl_types::f32_f32_f32_f32::from((origin.x, origin.y, origin.z, 0.0)),
        dir: glux::gl_types::f32_f32_f32_f32::from((dir.x, dir.y, dir.z, 0.0)),
    }
}
