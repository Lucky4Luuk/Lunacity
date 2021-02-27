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
const COMBINE_CS_PATH:    &str = "rt_lib/shaders/display_cs.glsl";

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct RawRayHit {
    pos: glux::gl_types::f32_f32_f32_f32,
    normal: glux::gl_types::f32_f32_f32_f32,
    dist: f32,
}

impl RawRayHit {
    pub fn empty() -> Self {
        Self {
            pos: glux::gl_types::f32_f32_f32_f32::new(0.0, 0.0, 0.0, 0.0),
            normal: glux::gl_types::f32_f32_f32_f32::new(0.0, 0.0, 0.0, 0.0),
            dist: 0.0,
        }
    }
}

pub struct Raytracer {
    raytrace_program: ShaderProgram,
    combine_program: ShaderProgram,
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

        let combine_cs_src = shader_processor::preprocessor(std::path::Path::new(COMBINE_CS_PATH));
        let combine_cs = Shader::from_source(&combine_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let combine_program = ShaderProgram::from_shader(&combine_cs);
        trace!("Combine shader loaded!");

        trace!("Raytracer loaded!");

        Self {
            raytrace_program: raytracing_program,
            output_program: output_program,
            combine_program: combine_program,

            dispatch_size: 32, //TODO: Connect this + workgroup size in shader together
        }
    }

    pub fn render(&self, camera: &Camera) {
        let inv_proj_view = (camera.get_projection_matrix(1280.0/720.0) * camera.get_view_matrix()).inverse();

        let ray_ssbo = glux::gl_types::ShaderStorageBuffer::new();
        let hit_ssbo = glux::gl_types::ShaderStorageBuffer::new();
        hit_ssbo.bind();
        hit_ssbo.data(&vec![RawRayHit::empty(); camera.resolution.0 * camera.resolution.1][..], gl::DYNAMIC_COPY);
        hit_ssbo.unbind();

        camera.generate_rays(&ray_ssbo);
        self.raytrace_program.bind();
        hit_ssbo.bind_buffer_base(0);
        ray_ssbo.bind_buffer_base(1);
        self.raytrace_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
        self.raytrace_program.uniform("invprojview", inv_proj_view);
        unsafe {
            gl::DispatchCompute(camera.resolution.0 as u32 / (self.dispatch_size-1), camera.resolution.1 as u32 / (self.dispatch_size-1), 1);
        }
        ray_ssbo.bind_buffer_base(0);
        self.raytrace_program.unbind();

        trace!("Raytracing done");

        unsafe {
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }

        //Combine hits
        self.combine_program.bind();
        self.combine_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
        hit_ssbo.bind_buffer_base(1);
        unsafe {
            gl::DispatchCompute(camera.resolution.0 as u32 / (self.dispatch_size-1), camera.resolution.1 as u32 / (self.dispatch_size-1), 1);
        }
        hit_ssbo.bind_buffer_base(0);
        self.combine_program.unbind();
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
