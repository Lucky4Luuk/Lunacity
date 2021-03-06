#[macro_use] extern crate log;

use std::time::Instant;

use glux::{
    Program, WindowSettings,
    mesh::{Vertex, Mesh},
};

use rt_lib::{
    Raytracer,
    objects::{
        Camera,
        Lambert,
    }
};

pub fn get_workgroup_invocations() -> i32 {
    let mut value = 0;
    unsafe {
        gl::GetIntegerv(gl::MAX_COMPUTE_WORK_GROUP_INVOCATIONS, &mut value);
    }
    value
}

fn main() {
    // let max_level = log::LevelFilter::max();
    let max_level = log::LevelFilter::Debug;
    pretty_env_logger::formatted_builder()
        .filter_level(max_level)
        .init();

    debug!("Hello, world!");

    let win_settings = WindowSettings {
        title: "[OPENGL] Lunacity - v0.0.1",
        resolution: (1280, 720),
        gl_version: (4, 5),
        vsync: false,
    };

    let mut program = Program::new(win_settings);

    // let dispatch_size = (get_workgroup_invocations() as f32).sqrt() as u32;
    let dispatch_size = (32, 30); //960, should be able to run on everything
    debug!("Dispatch size: {:?}", dispatch_size);

    let mut raytracer = Raytracer::new(dispatch_size);
    let lambert = Lambert;
    raytracer.add_brdf(&lambert);
    let mut camera = Camera::new((1280, 720), dispatch_size);

    let vertices: Vec<Vertex> = vec![
            Vertex {
                pos: (-1.0, -1.0, 0.0).into(),
                uv: (0.0, 0.0).into(),
                rgba: (1.0, 0.0, 0.0, 1.0).into(),
            },
            Vertex {
                pos: (1.0, -1.0, 0.0).into(),
                uv: (1.0, 0.0).into(),
                rgba: (0.0, 1.0, 0.0, 1.0).into(),
            },
            Vertex {
                pos: (1.0, 1.0, 0.0).into(),
                uv: (1.0, 1.0).into(),
                rgba: (0.0, 0.0, 1.0, 1.0).into(),
            },

            Vertex {
                pos: (-1.0, 1.0, 0.0).into(),
                uv: (0.0, 1.0).into(),
                rgba: (0.0, 1.0, 0.0, 1.0).into(),
            },
            Vertex {
                pos: (-1.0, -1.0, 0.0).into(),
                uv: (0.0, 0.0).into(),
                rgba: (1.0, 0.0, 0.0, 1.0).into(),
            },
            Vertex {
                pos: (1.0, 1.0, 0.0).into(),
                uv: (1.0, 1.0).into(),
                rgba: (0.0, 0.0, 1.0, 1.0).into(),
            },
        ];
    let quad = Mesh::from_vertices(&vertices);

    raytracer.render_sample(&mut camera);

    let mut last_frame = Instant::now();
    let mut total_time: f32 = 0.0;

    let mut event_pump = program.sdl_mut().event_pump().unwrap();
    'program: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'program,
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::A), timestamp, window_id, scancode, keymod, repeat } => {
                    let pixels = camera.get_texture_as_pixels();
                    println!("Pixels: {}", pixels.len());
                    image::save_buffer(&std::path::Path::new("test.png"), &pixels, 1280, 720, image::ColorType::Rgba8);
                    println!("Image saved!");
                },
                _ => {},
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        raytracer.render_sample(&mut camera);
        raytracer.test_output(&camera, &quad);

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;
        total_time += delta_s;

        program.sdl_window().gl_swap_window();
    }
}
