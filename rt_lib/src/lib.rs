#[macro_use] extern crate log;

use glux::{
    mesh::Mesh,
    shader::{Shader, ShaderProgram},
    gl_types::{Texture, f32_f32},
};

pub mod shader_processor;
pub mod objects;

use objects::{
    camera::Camera,
};

const PASSTHROUGH_VS_SRC: &str = include_str!("../shaders/passthrough_vs.glsl");
const PASSTHROUGH_FS_SRC: &str = include_str!("../shaders/passthrough_fs.glsl");

const RAYTRACING_CS_SRC:  &str = include_str!("../shaders/raytracing_cs.glsl");
const RAYTRACING_CS_PATH: &str = "rt_lib/shaders/raytracing_cs.glsl";

pub struct Raytracer {
    raytrace_program: ShaderProgram,
    output_program: ShaderProgram,

    render_buffer: Texture,
    resolution: (i32, i32),

    dispatch_size: u32,

    camera: Camera,
}

impl Raytracer {
    pub fn new(render_resolution: (i32, i32)) -> Self {
        let output_vs = Shader::from_source(PASSTHROUGH_VS_SRC, gl::VERTEX_SHADER).expect("Failed to compile shader!");
        let output_fs = Shader::from_source(PASSTHROUGH_FS_SRC, gl::FRAGMENT_SHADER).expect("Failed to compile shader!");
        let output_program = ShaderProgram::from_shaders(vec![&output_vs, &output_fs]);
        trace!("Output shader loaded!");

        let raytracing_cs_src = shader_processor::preprocessor(std::path::Path::new(RAYTRACING_CS_PATH));
        let raytracing_cs = Shader::from_source(&raytracing_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let raytracing_program = ShaderProgram::from_shader(&raytracing_cs);
        trace!("Raytracing shader loaded!");

        let texture = Texture::from_ptr(render_resolution, std::ptr::null(), gl::RGBA32F as i32, gl::RGBA);
        unsafe {
            gl::BindImageTexture(0, texture.id, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F);
        }
        trace!("Render texture constructed!");

        trace!("Raytracer loaded!");

        Self {
            raytrace_program: raytracing_program,
            output_program: output_program,

            render_buffer: texture,
            resolution: render_resolution,

            dispatch_size: 32, //TODO: Connect this + workgroup size in shader together

            camera: Camera::default(),
        }
    }

    pub fn render(&self) {
        let inv_proj_view = (self.camera.get_projection_matrix(1280.0/720.0) * self.camera.get_view_matrix()).inverse();
        self.raytrace_program.bind();
        self.raytrace_program.uniform("dims", f32_f32::from( (self.resolution.0 as f32, self.resolution.1 as f32) ));
        self.raytrace_program.uniform("invprojview", inv_proj_view);
        unsafe {
            gl::DispatchCompute(self.resolution.0 as u32 / (self.dispatch_size-1), self.resolution.1 as u32 / (self.dispatch_size-1), 1);
        }
        self.raytrace_program.unbind();
    }

    pub fn test_output(&self, mesh: &Mesh) {
        unsafe {
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }

        self.output_program.bind();
        self.render_buffer.bind();
        mesh.draw();
        self.render_buffer.unbind();
        self.output_program.unbind();
    }
}
