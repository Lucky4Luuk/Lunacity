#[macro_use] extern crate log;

use glux::{
    mesh::Mesh,
    shader::{Shader, ShaderProgram},
    gl_types::{Texture, f32_f32},
};

pub mod shader_processor;
pub mod objects;

use objects::{
    Camera,
};

const PASSTHROUGH_VS_SRC: &str = include_str!("../shaders/passthrough_vs.glsl");
const PASSTHROUGH_FS_SRC: &str = include_str!("../shaders/passthrough_fs.glsl");

const RAYTRACING_CS_PATH: &str = "rt_lib/shaders/raytracing_cs.glsl";

pub struct Raytracer {
    raytrace_program: ShaderProgram,
    output_program: ShaderProgram,

    dispatch_size: u32,
}

impl Raytracer {
    pub fn new() -> Self {
        let output_vs = Shader::from_source(PASSTHROUGH_VS_SRC, gl::VERTEX_SHADER).expect("Failed to compile shader!");
        let output_fs = Shader::from_source(PASSTHROUGH_FS_SRC, gl::FRAGMENT_SHADER).expect("Failed to compile shader!");
        let output_program = ShaderProgram::from_shaders(vec![&output_vs, &output_fs]);
        trace!("Output shader loaded!");

        let raytracing_cs_src = shader_processor::preprocessor(std::path::Path::new(RAYTRACING_CS_PATH));
        let raytracing_cs = Shader::from_source(&raytracing_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let raytracing_program = ShaderProgram::from_shader(&raytracing_cs);
        trace!("Raytracing shader loaded!");

        trace!("Raytracer loaded!");

        Self {
            raytrace_program: raytracing_program,
            output_program: output_program,

            dispatch_size: 32, //TODO: Connect this + workgroup size in shader together
        }
    }

    pub fn render(&self, camera: &Camera) {
        let inv_proj_view = (camera.get_projection_matrix(1280.0/720.0) * camera.get_view_matrix()).inverse();
        let ssbo = glux::gl_types::ShaderStorageBuffer::new();
        camera.generate_rays(&ssbo);
        self.raytrace_program.bind();
        ssbo.bind_buffer_base(1);
        self.raytrace_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
        self.raytrace_program.uniform("invprojview", inv_proj_view);
        unsafe {
            gl::DispatchCompute(camera.resolution.0 as u32 / (self.dispatch_size-1), camera.resolution.1 as u32 / (self.dispatch_size-1), 1);
        }
        ssbo.bind_buffer_base(0);
        self.raytrace_program.unbind();
    }

    pub fn test_output(&self, camera: &Camera, mesh: &Mesh) {
        unsafe {
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }

        self.output_program.bind();
        camera.render_buffer.bind();
        mesh.draw();
        camera.render_buffer.unbind();
        self.output_program.unbind();
    }
}
