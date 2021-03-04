#[macro_use] extern crate log;

use std::sync::Mutex;
use std::collections::HashMap;

use glux::gl_types::ShaderStorageBuffer;
use glux::{
    mesh::Mesh,
    shader::{Shader, ShaderProgram},
    gl_types::{Texture, f32_f32},
};

pub mod shader_processor;
pub mod objects;

use objects::{
    Camera,

    IsBRDF,
};

const PASSTHROUGH_VS_SRC: &str = include_str!("../shaders/passthrough_vs.glsl");
const PASSTHROUGH_FS_SRC: &str = include_str!("../shaders/passthrough_fs.glsl");

const RAYTRACING_CS_PATH: &str = "rt_lib/shaders/raytracing_cs.glsl";
const COMBINE_CS_PATH:    &str = "rt_lib/shaders/combine_cs.glsl";
const SHADING_CS_PATH:    &str = "rt_lib/shaders/shading_cs.glsl";
const WAVE_CS_PATH:       &str = "rt_lib/shaders/spawn_wave_cs.glsl";

fn string_to_id(input: String) -> u32 {
    let mut result = 0;
    let mut mult = 1;
    for c in input.chars() {
        result += c as u32 * mult;
        mult *= 10;
    }
    result
}

pub struct Raytracer {
    raytrace_program: ShaderProgram,
    shading_program: ShaderProgram,
    wave_program: ShaderProgram,
    combine_program: ShaderProgram,
    output_program: ShaderProgram,

    brdf_src: HashMap<String, (String, String)>,

    rng_ssbo: ShaderStorageBuffer,

    dispatch_size: (u32, u32),
    bounces: u32,
    samples: Mutex<u32>,
}

impl Raytracer {
    pub fn new(dispatch_size: (u32, u32)) -> Self {
        use rand::Rng;

        let output_vs = Shader::from_source(PASSTHROUGH_VS_SRC, gl::VERTEX_SHADER).expect("Failed to compile shader!");
        let output_fs = Shader::from_source(PASSTHROUGH_FS_SRC, gl::FRAGMENT_SHADER).expect("Failed to compile shader!");
        let output_program = ShaderProgram::from_shaders(vec![&output_vs, &output_fs]);
        debug!("Output shader loaded!");

        let raytracing_cs_src = shader_processor::preprocessor(std::path::Path::new(RAYTRACING_CS_PATH), dispatch_size);
        let raytracing_cs = Shader::from_source(&raytracing_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let raytracing_program = ShaderProgram::from_shader(&raytracing_cs);
        debug!("Raytracing shader loaded!");

        let shading_cs_src = shader_processor::preprocessor(std::path::Path::new(SHADING_CS_PATH), dispatch_size);
        let shading_cs = Shader::from_source(&shading_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let shading_program = ShaderProgram::from_shader(&shading_cs);
        debug!("Shading shader loaded!");

        let wave_cs_src = shader_processor::preprocessor(std::path::Path::new(WAVE_CS_PATH), dispatch_size);
        let wave_cs = Shader::from_source(&wave_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let wave_program = ShaderProgram::from_shader(&wave_cs);
        debug!("Wave spawn shader loaded!");

        let combine_cs_src = shader_processor::preprocessor(std::path::Path::new(COMBINE_CS_PATH), dispatch_size);
        let combine_cs = Shader::from_source(&combine_cs_src, gl::COMPUTE_SHADER).expect("Failed to compile shader!");
        let combine_program = ShaderProgram::from_shader(&combine_cs);
        debug!("Combine shader loaded!");

        let mut rng = rand::thread_rng();

        let rng_ssbo = glux::gl_types::ShaderStorageBuffer::new();
        let rng_vec: Vec<f32> = (0..1280*720).map(|_| rng.gen_range(0f32..2048f32)).collect();
        rng_ssbo.bind();
        rng_ssbo.data(&rng_vec[..], gl::DYNAMIC_COPY);
        rng_ssbo.unbind();
        debug!("RNG ssbo generated!");

        debug!("Raytracer loaded!");

        Self {
            raytrace_program: raytracing_program,
            shading_program: shading_program,
            wave_program: wave_program,
            combine_program: combine_program,
            output_program: output_program,

            brdf_src: HashMap::new(),

            rng_ssbo: rng_ssbo,

            dispatch_size: dispatch_size, //TODO: Connect this + workgroup size in shader together
            bounces: 4,
            samples: Mutex::new(0),
        }
    }

    /// Adds/updates a brdf in the shader.
    /// Warning: recompiles the entire shader.
    /// Not too heavy however to recompile.
    //TODO: reset the renderer, otherwise it'll use samples with a different BRDF
    pub fn add_brdf(&mut self, brdf: &dyn IsBRDF) {
        if !self.brdf_src.contains_key(&brdf.signature()) {
            self.brdf_src.insert(brdf.signature(), (brdf.signature(), brdf.code()));
        } else {
            todo!("Overwrite the old brdf");
        }
        self.update_brdf_general();
    }

    fn update_brdf_general(&mut self) {
        use std::path::Path;
        use std::fs::OpenOptions;
        use std::io::Write;

        //Update rt_lib/shaders/brdf/generated.glsl
        let path = Path::new("rt_lib/shaders/brdf/generated.glsl");

        let mut sel_func_src = String::from("vec3 material(int id, Material mat, vec3 light, vec3 view, vec3 normal, vec3 tangent, vec3 binormal) {\n"); //The function that selects a material based on index
        let mut full_src = String::from("#include \"mat.glsl\"\n#include \"lambert.glsl\"\n");
        for (name, (signature, code)) in &self.brdf_src {
            //Unique id from name
            let id = string_to_id(name.to_string());
            sel_func_src.push('\t'); //Insert tab for readability
            sel_func_src.push_str(&format!("if (id == {}) return {}(mat, light, view, normal, tangent, binormal);", id, signature));
            sel_func_src.push('\n');

            full_src.push_str(&code);
            full_src.push('\n');
        }

        full_src.push('\n');
        sel_func_src.push_str("\treturn builtin_lambert(mat, light, view, normal, tangent, binormal);
");
        sel_func_src.push('}');

        {
            let mut file = OpenOptions::new().write(true).open(path).expect("generated.glsl is missing!");
            file.set_len(0).unwrap();
            write!(&mut file, "{}", full_src);
            write!(&mut file, "{}", sel_func_src);
        }

        static DEFAULT_GENERATED_GLSL: &str = "//THIS FILE WILL GET CLEARED EVERYTIME YOU RUN THE RAYTRACER
//ANY MODIFICATIONS HERE ARE POINTLESS
//YOU HAVE BEEN WARNED

#include \"mat.glsl\"
#include \"lambert.glsl\"
vec3 material(int id, Material mat, vec3 light, vec3 view, vec3 normal, vec3 tangent, vec3 binormal) {
	return builtin_lambert(mat, light, view, normal, tangent, binormal);
}";

        let shading_cs_src = shader_processor::preprocessor(std::path::Path::new(SHADING_CS_PATH), self.dispatch_size);
        let shading_cs = match Shader::from_source(&shading_cs_src, gl::COMPUTE_SHADER) {
            Ok(cs) => cs,
            Err(_) => {
                let mut file = OpenOptions::new().write(true).open(path).expect("generated.glsl is missing!");
                file.set_len(0).unwrap();
                write!(&mut file, "{}", DEFAULT_GENERATED_GLSL);
                panic!("Failed to compile shader!");
            }
        };
        let shading_program = ShaderProgram::from_shader(&shading_cs);
        debug!("Shading shader reloaded!");
        self.shading_program = shading_program;

        let wave_cs_src = shader_processor::preprocessor(std::path::Path::new(WAVE_CS_PATH), self.dispatch_size);
        println!("{}", wave_cs_src);
        let wave_cs = match Shader::from_source(&wave_cs_src, gl::COMPUTE_SHADER) {
            Ok(cs) => cs,
            Err(_) => {
                let mut file = OpenOptions::new().write(true).open(path).expect("generated.glsl is missing!");
                file.set_len(0).unwrap();
                write!(&mut file, "{}", DEFAULT_GENERATED_GLSL);
                panic!("Failed to compile shader!");
            }
        };
        let wave_program = ShaderProgram::from_shader(&wave_cs);
        debug!("Wave spawn shader reloaded!");
        self.wave_program = wave_program;

        {
            let mut file = OpenOptions::new().write(true).open(path).expect("generated.glsl is missing!");
            file.set_len(0).unwrap();
            write!(&mut file, "{}", DEFAULT_GENERATED_GLSL);
        }
    }

    pub fn render_sample(&self, camera: &Camera) {
        use rand::Rng;

        // trace!("RNG ssbo updated!");
        // trace!("Time since start: {:?}", Instant::now() - func_start);

        camera.generate_rays(self.dispatch_size);
        camera.clear_sample_texture();

        // trace!("Rays generated and sample texture cleared!");
        // trace!("Time since start: {:?}", Instant::now() - func_start);

        {
            let samples = *self.samples.lock().unwrap();
            if samples % 32 == 0 {
                // let mut rng = rand::thread_rng();
                // let rng_vec: Vec<f32> = (0..128).map(|_| rng.gen_range(1f32..2048f32)).collect();
                // self.rng_ssbo.bind();
                // self.rng_ssbo.data(&rng_vec[..], gl::DYNAMIC_COPY);
                // self.rng_ssbo.unbind();
                debug!("Refreshed RNG buffer! Samples: {}", samples);
            }
        }

        for _i in 0..self.bounces {
            self.raytrace_program.bind();
            camera.hit_ssbo.bind_buffer_base(0);
            camera.ray_ssbo.bind_buffer_base(1);
            self.raytrace_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
            unsafe {
                gl::DispatchCompute(camera.resolution.0 as u32 / self.dispatch_size.0, camera.resolution.1 as u32 / self.dispatch_size.1, 1);
            }
            camera.ray_ssbo.bind_buffer_base(0);
            self.raytrace_program.unbind();

            unsafe {
                gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
            }

            //Generate new rays from hits
            self.wave_program.bind();
            self.wave_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
            self.wave_program.uniform("samples", *self.samples.lock().unwrap() as f32);
            camera.hit_ssbo.bind_buffer_base(0);
            self.rng_ssbo.bind_buffer_base(1);
            camera.ray_ssbo.bind_buffer_base(2);
            unsafe {
                gl::DispatchCompute(camera.resolution.0 as u32 / self.dispatch_size.0, camera.resolution.1 as u32 / self.dispatch_size.1, 1);
            }
            camera.hit_ssbo.bind_buffer_base(0);
            self.rng_ssbo.bind_buffer_base(0);
            camera.ray_ssbo.bind_buffer_base(0);
            self.wave_program.unbind();

            //Shade hits and output to texture
            self.shading_program.bind();
            camera.bind_sample_texture(0);
            self.shading_program.uniform("dims", f32_f32::from( (camera.resolution.0 as f32, camera.resolution.1 as f32) ));
            camera.hit_ssbo.bind_buffer_base(1);
            unsafe {
                gl::DispatchCompute(camera.resolution.0 as u32 / self.dispatch_size.0, camera.resolution.1 as u32 / self.dispatch_size.1, 1);
            }
            camera.hit_ssbo.bind_buffer_base(0);
            self.shading_program.unbind();
        }

        let mut samples_lock = self.samples.lock().unwrap();
        *samples_lock += 1;

        let samples: i32 = (*samples_lock) as i32;

        // println!("Samples: {}", samples);

        self.combine_program.bind();
        self.combine_program.uniform("samples", samples);
        camera.bind_sample_texture(0);
        camera.bind_final_texture(1);
        unsafe {
            gl::DispatchCompute(camera.resolution.0 as u32 / self.dispatch_size.0, camera.resolution.1 as u32 / self.dispatch_size.1, 1);
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
