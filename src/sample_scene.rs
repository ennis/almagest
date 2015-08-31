
use frame;
use buffer;
use context;
use shader;
use attrib;
use draw;
use render_target;
use render_queue;
use texture;
use glutil;
use fern;
use log;
use time;
use std;

use num::traits::{Zero};
use nalgebra::*;
use std::sync::mpsc::{Receiver};
use glfw;
use glfw::{Context, Key, Window, WindowEvent};
use gl;
use gl::types::*;
use libc::{c_void};
use std::ffi::{CString, CStr};
use std::cell::{RefCell};
use std::mem;
use std::path::{Path};
use image;
use image::GenericImage;
use input;
use input::{Event, InputEvent};
use camera::{Camera, TrackballCameraSettings, TrackballCameraController};
use glutil::MeshVertex;

static VS_SOURCE : &'static str = r#"
#version 440
layout (binding = 0) uniform ShaderParams
{
	mat4 viewMatrix;
	mat4 projMatrix;
	vec3 uColor;
};
in vec3 position;
out vec2 tc;
void main() {
	vec4 temp_pos = projMatrix * viewMatrix * vec4(position, 1.0);
	gl_Position = temp_pos;
	//gl_Position = vec4(position.xy, 0.0, 1.0);
	tc = temp_pos.xy;
}
"#;

static FS_SOURCE : &'static str = r#"
#version 440
layout (binding = 0) uniform ShaderParams
{
	mat4 viewMatrix;
	mat4 projMatrix;
	vec3 uColor;
};

layout (binding = 0) uniform sampler2D tex0;
in vec2 tc;
out vec4 color;
void main() {
	color = vec4(uColor, 1.0f);
}
"#;

#[repr(C)]
#[derive(Copy, Clone)]
struct ShaderParams
{
	u_color: Vec3<f32>
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ShaderParams2
{
	view_mat: Mat4<f32>,
	proj_mat: Mat4<f32>,
	u_color: Vec3<f32>,
}


fn make_circle(radius: f32, divisions: u16) -> 
	(Vec<glutil::MeshVertex>, Vec<u16>)
{
	let nullvec = Vec3::<f32>::zero();
	assert!(divisions >= 2);
	let mut result = Vec::with_capacity((1 + divisions) as usize);
	let mut result_indices = Vec::with_capacity((divisions * 3) as usize);
	result.push(glutil::MeshVertex {
		pos: nullvec,
		norm: nullvec, 
		tg: nullvec });
	for i in 0..divisions
	{
		let th = ((i as f32) / (divisions as f32)) * 2.0f32 * std::f32::consts::PI;
		result.push(glutil::MeshVertex {
			pos: Vec3::new(radius * f32::cos(th), radius * f32::sin(th), 0.0f32), 
			norm: nullvec, 
			tg: nullvec });
		result_indices.push(0u16);
		result_indices.push(i+1);
		result_indices.push(if i == divisions-1 { 1 } else { i+2 });
	}
	(result, result_indices)
}


// Input state
/*pub enum InputState
{

}*/

// convert GLFW event loop
fn event_loop<F: FnMut(Event, &glfw::Window) -> bool>(
	glfw: &mut glfw::Glfw, 
	w: &mut glfw::Window, 
	events: &Receiver<(f64, WindowEvent)>, 
	mut event_handler: F)
{
	 while !w.should_close() {
	 	// Translate input events
	    glfw.poll_events();
	    for (_, event) in glfw::flush_messages(&events) {
	        match event {
		        glfw::WindowEvent::Key(Key::Escape, _, glfw::Action::Press, _) => {
		            w.set_should_close(true);
		        },
		        glfw::WindowEvent::Key(k, _, glfw::Action::Press, _) => {
		        	event_handler(Event::Input(InputEvent::KeyEvent(k)), w);
		        },
		        _ => {}
		    }
	    }
	    // send render event
	    event_handler(Event::Update(0.0f32), w);
	    event_handler(Event::Render(0.0f32), w);
	    w.swap_buffers();
	}
}


pub fn sample_scene() 
{

	let mesh_data = [
		glutil::MeshVertex { pos: Vec3::new(0.0f32, 0.0f32, 1.0f32), norm: Vec3::new(0.0f32, 0.0f32, 0.0f32), tg: Vec3::<f32>::zero() },
		glutil::MeshVertex { pos: Vec3::new(1.0f32, 1.0f32, 1.0f32), norm: Vec3::new(0.0f32, 0.0f32, 0.0f32), tg: Vec3::<f32>::zero() },
		glutil::MeshVertex { pos: Vec3::new(0.0f32, 1.0f32, 1.0f32), norm: Vec3::new(0.0f32, 0.0f32, 0.0f32), tg: Vec3::<f32>::zero() }];

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
	let (mut window, events) = glfw.create_window(640, 640, "Hello this is window", glfw::WindowMode::Windowed)
	        .expect("Failed to create GLFW window.");
	window.set_key_polling(true);
	window.make_current();

	// Load GL function pointers
	gl::load_with(|s| window.get_proc_address(s));

	let mut vao: GLuint = 0;
	unsafe { 
		gl::GenVertexArrays(1, &mut vao); 
		gl::BindVertexArray(vao);
		// attrib #1 position
		gl::EnableVertexAttribArray(0);
		gl::VertexAttribFormat(0, 3, gl::FLOAT, gl::FALSE, 12);
		gl::VertexAttribBinding(0, 0);
	}

	let ctx = context::Context::new();
	let prog = ctx.create_program_from_source(VS_SOURCE, FS_SOURCE).expect("Error creating program");

	let (mesh2_vertex, mesh2_indices) = make_circle(1.0f32, 400);
	let mesh2 = glutil::Mesh::new(
		&ctx, draw::PrimitiveType::Triangle, 
		&mesh2_vertex,
		Some(&mesh2_indices));

	let mesh = glutil::Mesh::new(
		&ctx, 
		draw::PrimitiveType::Triangle, 
		&mesh_data, 
		None);

	let cube_mesh = glutil::Mesh::new(
		&ctx,
		draw::PrimitiveType::Triangle,
		&cube_vertex_data,
		Some(&cube_index_data));
	
	let layout = attrib::InputLayout::new(1, &[
		attrib::Attribute{ slot: 0, ty: attrib::AttributeType::Float3 },
		attrib::Attribute{ slot: 0, ty: attrib::AttributeType::Float3 },
		attrib::Attribute{ slot: 0, ty: attrib::AttributeType::Float3 }]);

	//-------------------------------
	// image test
	let img = image::open(&Path::new("test.jpg")).unwrap();
	let (w, h) = img.dimensions();
	let img2 = img.as_rgb8().unwrap();
	let tex = texture::Texture2D::new(w, h, 1, texture::TextureFormat::Unorm8x3, Some(img2));

	unsafe {
		let mut sampler : GLuint = 0;
		gl::GenSamplers(1, &mut sampler);
		gl::SamplerParameteri(sampler, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
		gl::SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
		gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_R, gl::REPEAT as i32);
		gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
		gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
		gl::BindSampler(0, sampler);
		tex.bind(0);
	}

	let mut camera_controller = TrackballCameraSettings::default().build();


	event_loop(&mut glfw, &mut window, &events, |event, window| {
		match event {
			Event::Render(dt) => {
				// update camera
				let cam = camera_controller.get_camera(window);
				let shader_params = ShaderParams2 { 
					view_mat: cam.view_matrix, 
					proj_mat: cam.proj_matrix, 
					u_color: Vec3::new(0.0f32, 1.0f32, 0.0f32)};

				// begin frame
			    unsafe {
					gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
					gl::ClearColor(1.0f32, 0.0f32, 0.0f32, 0.0f32);
					gl::ClearDepth(1.0);
					gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
					gl::Disable(gl::DEPTH_TEST);
					gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
			    }

				let frame = ctx.create_frame(render_target::RenderTarget::Screen);

				// put a scope here to end borrow of 'frame' before handing it to ctx
				{
					// allocate temp buffer for shader parameters
					//let param_buf = frame.make_uniform_buffer(&ShaderParams {u_color: Vec3::new(0.0f32, 1.0f32, 0.0f32)});
					// allocate another for the lulz
					//let param_buf_2 = frame.make_uniform_buffer(&ShaderParams {u_color: Vec3::new(0.0f32, 1.0f32, 0.0f32)});
					let param_buf_3 = frame.make_uniform_buffer(&shader_params);


					// render queue test
					let mut rq = render_queue::RenderQueue::new();
					{
						let mesh_part = draw::MeshPart { 
							primitive_type: draw::PrimitiveType::Triangle,
							start_vertex: 0,
							start_index: 0,
							num_vertices: mesh2.num_vertices as u32,
							num_indices: mesh2.num_indices as u32 };

						let mat_block = rq.create_material_block(&prog, &[]);
						let mat_block_2 = rq.create_material_block(&prog, &[]);

						let vertex_block = rq.create_vertex_input_block(
							&layout,
							&[buffer::Binding{ slot: 0, slice: mesh2.vb.raw.as_raw_buf_slice()}], None);
						let vertex_block_2 = rq.create_vertex_input_block(
							&layout,
							&[buffer::Binding{ slot: 0, slice: mesh2.vb.raw.as_raw_buf_slice()}], 
							if let Some(ref ib) = mesh2.ib { Some(ib.raw.as_raw_buf_slice()) } else { None });

						let vertex_block_3 = rq.create_vertex_input_block(
							&layout,
							&[buffer::Binding{ slot: 0, slice: cube_mesh.vb.raw.as_raw_buf_slice()}], 
							Some(cube_mesh.ib.as_ref().unwrap().raw.as_raw_buf_slice()));
						
						rq.add_render_item(mat_block, vertex_block_3, mesh_part, Some(buffer::Binding{slot: 0, slice: param_buf_3.as_raw()}));
						rq.add_render_item(mat_block, vertex_block_2, mesh_part, Some(buffer::Binding{slot: 0, slice: param_buf_3.as_raw()}));
						//rq.add_render_item(mat_block, vertex_block, mesh_part, Some(buffer::Binding{slot: 0, slice: param_buf_2.as_raw()}));
						rq.execute(&frame, None);
					}
					rq.clear();
				}
				ctx.commit_frame(frame);
			},

			_ => {}
		};

		true
	});
}