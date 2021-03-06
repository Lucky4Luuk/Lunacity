use glux::gl_types::f32_f32;
use glux::shader::ShaderProgram;
use glux::shader::Shader;
use glux::gl_types::ShaderStorageBuffer;

use glux::gl_types::Texture;

use glam::*;

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Ray {
    pos:      glux::gl_types::f32_f32_f32_f32,
    dir:      glux::gl_types::f32_f32_f32_f32,
    pixel:    glux::gl_types::f32_f32_f32_f32,
    power:    glux::gl_types::f32_f32_f32_f32,
}

impl Ray {
    pub fn default() -> Self {
        Self {
            pos:      glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,0.0),
            dir:      glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,0.0),
            pixel:    glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,0.0),
            power:    glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,0.0),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct RawRayHit {
    pos:         glux::gl_types::f32_f32_f32_f32,
    normal_dist: glux::gl_types::f32_f32_f32_f32,
    pixel:       glux::gl_types::f32_f32_f32_f32,
    dir_pow:     glux::gl_types::f32_f32_f32_f32,
    power:       glux::gl_types::f32_f32_f32_f32,
}

impl RawRayHit {
    pub fn empty() -> Self {
        Self {
            pos:         glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,0.0),
            normal_dist: glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,0.0),
            pixel:       glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,0.0),
            dir_pow:     glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,1.0),
            power:       glux::gl_types::f32_f32_f32_f32::new(0.0,0.0,0.0,1.0),
        }
    }
}

const RAY_CS_PATH: &str = "rt_lib/shaders/camera_ray_cs.glsl";

pub struct Camera {
    pub eye: Vec3,
    pub look_at: Vec3,

    pub fov: f32, //In degrees

    pub resolution: (usize, usize), //Output resolution
    pub sample_buffer: Texture,     //Output buffer for current sample
    pub render_buffer: Texture,     //Output buffer for final result

    pub ray_ssbo: ShaderStorageBuffer,
    pub hit_ssbo: ShaderStorageBuffer,
    pub jitter: f32_f32,

    pub ray_program: ShaderProgram,
}

impl Camera {
    pub fn new(resolution: (usize, usize), dispatch_size: (u32, u32)) -> Self {
        let sample_texture = Texture::from_ptr((resolution.0 as i32, resolution.1 as i32), std::ptr::null(), gl::RGBA32F as i32, gl::RGBA);
        trace!("Sample texture constructed!");

        let output_texture = Texture::from_ptr((resolution.0 as i32, resolution.1 as i32), std::ptr::null(), gl::RGBA32F as i32, gl::RGBA);
        trace!("Render texture constructed!");

        let ray_ssbo = glux::gl_types::ShaderStorageBuffer::new();

        let hit_ssbo = ShaderStorageBuffer::new();
        hit_ssbo.bind();
        hit_ssbo.data(&vec![RawRayHit::empty(); resolution.0 * resolution.1][..], gl::DYNAMIC_COPY);
        hit_ssbo.unbind();

        let ray_cs_src = crate::shader_processor::preprocessor(std::path::Path::new(RAY_CS_PATH), dispatch_size);
        let ray_cs = Shader::from_source(&ray_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let ray_program = ShaderProgram::from_shader(&ray_cs);
        trace!("Combine shader loaded!");

        let mut camera = Self {
            eye: vec3(0.0,0.0,-5.0),
            look_at: vec3(0.0,0.0,0.0),

            fov: 60.0,

            resolution: resolution,
            sample_buffer: sample_texture,
            render_buffer: output_texture,

            ray_ssbo: ray_ssbo,
            hit_ssbo: hit_ssbo,
            jitter: (0.0, 0.0).into(),

            ray_program: ray_program,
        };

        camera.generate_rays(dispatch_size);

        camera
    }

    //TODO: Implement this in glux for textures, so we don't have to wrap it here
    pub fn clear_sample_texture(&self) {
        unsafe {
            gl::ClearTexImage(self.sample_buffer.id, 0, gl::RGBA, gl::UNSIGNED_BYTE, std::ptr::null());
        }
    }

    pub fn bind_sample_texture(&self, id: u32) {
        unsafe {
            gl::BindImageTexture(id, self.sample_buffer.id, 0, gl::FALSE, 0, gl::READ_WRITE, gl::RGBA32F);
        }
    }

    pub fn bind_final_texture(&self, id: u32) {
        unsafe {
            gl::BindImageTexture(id, self.render_buffer.id, 0, gl::FALSE, 0, gl::READ_WRITE, gl::RGBA32F);
        }
    }

    pub fn get_texture_as_pixels(&self) -> Vec<u8> {
        self.render_buffer.bind();
        let mut pixels = vec![0u8; self.resolution.0 * self.resolution.1 * 4];
        unsafe {
            gl::GetTexImage(gl::TEXTURE_2D, 0, gl::RGBA, gl::UNSIGNED_BYTE, pixels.as_mut_ptr() as *mut std::ffi::c_void);
        }
        self.render_buffer.unbind();

        return pixels;
    }

    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov / 180.0 * std::f32::consts::PI, aspect_ratio, 0.02, 1024.0)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.look_at, vec3(0.0,1.0,0.0))
    }

    //TODO: Move ray generation to a shader
    pub fn generate_rays(&mut self, dispatch_size: (u32, u32)) {
        use rand::Rng;

        let inv_proj_view = (self.get_projection_matrix(self.resolution.0 as f32 / self.resolution.1 as f32) * self.get_view_matrix()).inverse();

        let data = vec![Ray::default(); self.resolution.0 * self.resolution.1];

        self.ray_ssbo.bind();
        self.ray_ssbo.data(&data[..], gl::DYNAMIC_COPY);
        self.ray_ssbo.unbind();

        let mut rng = rand::thread_rng();
        self.jitter.d0 = rng.gen::<f32>();
        self.jitter.d1 = rng.gen::<f32>();

        self.ray_program.bind();
        self.ray_ssbo.bind_buffer_base(0);
        self.ray_program.uniform("dims", f32_f32::from( (self.resolution.0 as f32, self.resolution.1 as f32) ));
        self.ray_program.uniform("invprojview", inv_proj_view);
        self.ray_program.uniform("jitter", self.jitter);
        unsafe {
            gl::DispatchCompute(self.resolution.0 as u32 / dispatch_size.0, self.resolution.1 as u32 / dispatch_size.1, 1);
        }
        self.ray_program.unbind();
    }
}
