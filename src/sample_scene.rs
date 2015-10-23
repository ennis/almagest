
use fern;
use log;
use time;
use std;

use rendering as rd;
use num::traits::{Zero};
use nalgebra::*;
use glfw;
use glfw::{Context, Key, WindowEvent};
use gl;
use gl::types::*;
use libc::{c_void};
use std::ffi::{CString, CStr};
use std::cell::{RefCell};
use std::mem;
use std::path::{Path, PathBuf};
use image;
use image::GenericImage;
use event;
use event::{Event, Input};
use camera::*;
use window::*;
use scene::*;
use material::*;
use scene_data::*;
use terrain::*;
use asset_loader::*;
use std::collections::HashMap;
use std::rc::Rc;
use graphics::*;
use player::*;

use std::io::{BufRead};

#[repr(C)]
#[derive(Copy, Clone)]
struct ShaderParams
{
	u_color: Vec3<f32>
}


fn make_circle(radius: f32, divisions: u16) ->
	(Vec<MeshVertex>, Vec<u16>)
{
	assert!(divisions >= 2);
	let mut result = Vec::with_capacity((1 + divisions) as usize);
	let mut result_indices = Vec::with_capacity((divisions * 3) as usize);
	result.push(MeshVertex {
		pos: [0.0; 3],
		norm: [0.0; 3],
		tg: [0.0; 3],
		tex: [0.0; 2]  });
	for i in 0..divisions
	{
		let th = ((i as f32) / (divisions as f32)) * 2.0 * std::f32::consts::PI;
		result.push(MeshVertex {
			pos: [radius*f32::cos(th), radius*f32::sin(th), 0.0f32],
			norm: [0.0; 3],
			tg: [0.0; 3],
			tex: [0.0; 2]  });
		result_indices.push(0u16);
		result_indices.push(i+1);
		result_indices.push(if i == divisions-1 { 1 } else { i+2 });
	}
	(result, result_indices)
}


pub fn sample_scene()
{
	// Cube

    let cube_vertex_data = [
        //top (0, 0, 1)
        MeshVertex::new([-1.0, -1.0,  1.0], [0.0, 0.0]),
        MeshVertex::new([ 1.0, -1.0,  1.0], [1.0, 0.0]),
        MeshVertex::new([ 1.0,  1.0,  1.0], [1.0, 1.0]),
        MeshVertex::new([-1.0,  1.0,  1.0], [0.0, 1.0]),
        //bottom (0.0, 0.0, -1.0)
        MeshVertex::new([ 1.0,  1.0, -1.0], [0.0, 0.0]),
        MeshVertex::new([-1.0,  1.0, -1.0], [1.0, 0.0]),
        MeshVertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        MeshVertex::new([ 1.0, -1.0, -1.0], [0.0, 1.0]),
        //right (1.0, 0.0, 0.0)
        MeshVertex::new([ 1.0, -1.0, -1.0], [0.0, 0.0]),
        MeshVertex::new([ 1.0,  1.0, -1.0], [1.0, 0.0]),
        MeshVertex::new([ 1.0,  1.0,  1.0], [1.0, 1.0]),
        MeshVertex::new([ 1.0, -1.0,  1.0], [0.0, 1.0]),
        //left (-1.0, 0.0, 0.0)
        MeshVertex::new([-1.0,  1.0,  1.0], [0.0, 0.0]),
        MeshVertex::new([-1.0, -1.0,  1.0], [1.0, 0.0]),
        MeshVertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        MeshVertex::new([-1.0,  1.0, -1.0], [0.0, 1.0]),
        //front (0.0, 1.0, 0.0)
        MeshVertex::new([-1.0,  1.0, -1.0], [0.0, 0.0]),
        MeshVertex::new([ 1.0,  1.0, -1.0], [1.0, 0.0]),
        MeshVertex::new([ 1.0,  1.0,  1.0], [1.0, 1.0]),
        MeshVertex::new([-1.0,  1.0,  1.0], [0.0, 1.0]),
        //back (0.0, -1.0, 0.0)
        MeshVertex::new([ 1.0, -1.0,  1.0], [0.0, 0.0]),
        MeshVertex::new([-1.0, -1.0,  1.0], [1.0, 0.0]),
        MeshVertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        MeshVertex::new([ 1.0, -1.0, -1.0], [0.0, 1.0]),
    ];

    let cube_index_data = [
         0,  1,  2,  2,  3,  0, // top
         4,  6,  5,  6,  4,  7, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 14, 13, 14, 12, 16, // left
        16, 18, 17, 18, 16, 19, // front
        20, 21, 22, 22, 23, 20, // back
    ];

	//-------------------------------
	// LOGGING
	let logger_config = fern::DispatchConfig {
	    format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
	        format!("[{}][{}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
	    }),
	    output: vec![fern::OutputConfig::stdout(), fern::OutputConfig::file("output.log")],
	    level: log::LogLevelFilter::Trace,
	};
	if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
	    panic!("Failed to initialize global logger: {}", e);
	}

	//-------------------------------
	// GLFW
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
	glfw.window_hint(glfw::WindowHint::Samples(4));

	let mut win = WindowSettings::new("ALMAGEST", (1024, 768)).build(&glfw).expect("Failed to create GLFW window.");
	let mut input = Input::new();

	// default sampler
	unsafe {
		let mut sampler : GLuint = 0;
		gl::GenSamplers(1, &mut sampler);
		gl::SamplerParameteri(sampler, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
		gl::SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
		gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
		gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
		gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
		gl::BindSampler(0, sampler);
		gl::BindSampler(1, sampler);
		//tex.bind(0);
	}

	let mut ctx = rd::Context::new();

	let (mesh2_vertex, mesh2_indices) = make_circle(1.0f32, 400);
	let mesh2 = Mesh::new(
		&ctx, rd::PrimitiveType::Triangle,
		&mesh2_vertex,
		Some(&mesh2_indices));

	let cube_mesh = Mesh::new(
		&ctx,
		rd::PrimitiveType::Triangle,
		&cube_vertex_data,
		Some(&cube_index_data));

	let banana_mesh = Mesh::load_from_obj(
		&ctx,
		Path::new("assets/models/banana.obj"),
		);

	let mut camera_controller = TrackballCameraSettings::default().build();
	let graphics = Graphics::new(&ctx);
	let terrain_renderer = TerrainRenderer::new();
	// load sample scene
	let mut scene = Scene::load(
		&ctx,
		&Path::new("assets"),
		&Path::new("assets/scenes/scene.json"));

	let mut tcur = 0.0f64;
	let mut tlast =  time::precise_time_s();
	win.event_loop(&mut glfw, |event, window| {
		tcur = time::precise_time_s();
		let dt = tcur - tlast;
		tlast = tcur;

		ctx.event(&event);
		input.event(&event);
		camera_controller.event(&event);
		scene.event(&event);
		scene.update(dt, &input);

		let cam = camera_controller.get_camera(window);

		match event {
			Event::Render(dt) => {
				scene.render(&graphics, &terrain_renderer, window, &ctx, &cam);
			},

			_ => {}
		};

		true
	});

}
