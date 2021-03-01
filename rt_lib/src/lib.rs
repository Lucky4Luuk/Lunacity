#[macro_use] extern crate log;

use std::sync::Mutex;
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
const COMBINE_CS_PATH:    &str = "rt_lib/shaders/combine_cs.glsl";
const SHADING_CS_PATH:    &str = "rt_lib/shaders/shading_cs.glsl";
const WAVE_CS_PATH:       &str = "rt_lib/shaders/spawn_wave_cs.glsl";

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct RawRayHit {
    pos: glux::gl_types::f32_f32_f32_f32,
    normal_dist: glux::gl_types::f32_f32_f32_f32,
}

impl RawRayHit {
    pub fn empty() -> Self {
        Self {
            pos: glux::gl_types::f32_f32_f32_f32::new(0.0, 0.0, 0.0, 0.0),
            normal_dist: glux::gl_types::f32_f32_f32_f32::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

pub struct Raytracer {
    raytrace_program: ShaderProgram,
    shading_program: ShaderProgram,
    wave_program: ShaderProgram,
    combine_program: ShaderProgram,
    output_program: ShaderProgram,

    dispatch_size: u32,
    bounces: u32,
    samples: Mutex<u32>,
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

        let shading_cs_src = shader_processor::preprocessor(std::path::Path::new(SHADING_CS_PATH));
        let shading_cs = Shader::from_source(&shading_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let shading_program = ShaderProgram::from_shader(&shading_cs);
        trace!("Shading shader loaded!");

        let wave_cs_src = shader_processor::preprocessor(std::path::Path::new(WAVE_CS_PATH));
        let wave_cs = Shader::from_source(&wave_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let wave_program = ShaderProgram::from_shader(&wave_cs);
        trace!("Wave spawn shader loaded!");

        let combine_cs_src = shader_processor::preprocessor(std::path::Path::new(COMBINE_CS_PATH));
        let combine_cs = Shader::from_source(&combine_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let combine_program = ShaderProgram::from_shader(&combine_cs);
        trace!("Combine shader loaded!");

        trace!("Raytracer loaded!");

        Self {
            raytrace_program: raytracing_program,
            shading_program: shading_program,
            wave_program: wave_program,
            combine_program: combine_program,
            output_program: output_program,

            dispatch_size: 32, //TODO: Connect this + workgroup size in shader together
            bounces: 2,
            samples: Mutex::new(0),
        }
    }

    pub fn render_sample(&self, camera: &Camera) {
        use rand::Rng;
        use std::time::Instant;

        let func_start = Instant::now();

        let mut rng = rand::thread_rng();

        let hit_ssbo = glux::gl_types::ShaderStorageBuffer::new();
        hit_ssbo.bind();
        hit_ssbo.data(&vec![RawRayHit::empty(); camera.resolution.0 * camera.resolution.1][..], gl::DYNAMIC_COPY);
        hit_ssbo.unbind();

        trace!("Hit ssbo created!");
        trace!("Time since start: {:?}", Instant::now() - func_start);

        let rng_ssbo = glux::gl_types::ShaderStorageBuffer::new();
        let rng_vec: Vec<f32> = (0..(camera.resolution.0*camera.resolution.1)).map(|_| rng.gen::<f32>()).collect();
        rng_ssbo.bind();
        rng_ssbo.data(&rng_vec[..], gl::DYNAMIC_COPY);
        rng_ssbo.unbind();

        trace!("RNG ssbo created!");
        trace!("Time since start: {:?}", Instant::now() - func_start);

        camera.generate_rays();
        camera.clear_sample_texture();

        trace!("Rays generated and sample texture cleared!");
        trace!("Time since start: {:?}", Instant::now() - func_start);

        for _i in 0..self.bounces {
            self.raytrace_program.bind();
            hit_ssbo.bind_buffer_base(0);
            camera.ray_ssbo.bind_buffer_base(1);
            self.raytrace_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
            unsafe {
                gl::DispatchCompute(camera.resolution.0 as u32 / (self.dispatch_size-1), camera.resolution.1 as u32 / (self.dispatch_size-1), 1);
            }
            camera.ray_ssbo.bind_buffer_base(0);
            self.raytrace_program.unbind();

            unsafe {
                gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
            }

            //Shade hits and output to texture
            self.shading_program.bind();
            camera.bind_sample_texture(0);
            self.shading_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
            hit_ssbo.bind_buffer_base(1);
            unsafe {
                gl::DispatchCompute(camera.resolution.0 as u32 / (self.dispatch_size-1), camera.resolution.1 as u32 / (self.dispatch_size-1), 1);
            }
            hit_ssbo.bind_buffer_base(0);
            self.shading_program.unbind();

            //Generate new rays from hits
            self.wave_program.bind();
            self.wave_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
            hit_ssbo.bind_buffer_base(0);
            rng_ssbo.bind_buffer_base(1);
            camera.ray_ssbo.bind_buffer_base(2);
            unsafe {
                gl::DispatchCompute(camera.resolution.0 as u32 / (self.dispatch_size-1), camera.resolution.1 as u32 / (self.dispatch_size-1), 1);
            }
            hit_ssbo.bind_buffer_base(0);
            rng_ssbo.bind_buffer_base(0);
            camera.ray_ssbo.bind_buffer_base(0);
            self.wave_program.unbind();
        }

        let mut samples_lock = self.samples.lock().unwrap();
        *samples_lock += 1;

        let samples: i32 = (*samples_lock) as i32;
        println!("Samples: {}", samples);

        self.combine_program.bind();
        self.combine_program.uniform("samples", samples);
        camera.bind_sample_texture(0);
        camera.bind_final_texture(1);
        unsafe {
            gl::DispatchCompute(camera.resolution.0 as u32 / (self.dispatch_size-1), camera.resolution.1 as u32 / (self.dispatch_size-1), 1);
        }
        self.combine_program.unbind();
    }

    pub fn test_output(&self, camera: &Camera, mesh: &Mesh) {
        unsafe {
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }

        self.output_program.bind();
        camera.render_buffer.bind();
        // camera.sample_buffer.bind();
        mesh.draw();
        camera.render_buffer.unbind();
        self.output_program.unbind();
    }
}
