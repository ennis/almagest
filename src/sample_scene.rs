
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
use event::{Event};
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

struct AssetCache<T>
{
	map: RefCell<HashMap<String, Rc<T>>>
}

impl<T> AssetCache<T>
{
	pub fn new() -> AssetCache<T>
	{
		AssetCache {
			map: RefCell::new(HashMap::new())
		}
	}

	fn load_with<F: Fn(&str) -> T>(&self, id: &str, f: F) -> Rc<T>
	{
		//self.map.entry(id).or_insert_with(Rc::new(f(id))).clone();
		let key_found = self.map.borrow().contains_key(id);
		if !key_found {
			let val = Rc::new(f(id));
			self.map.borrow_mut().insert(id.to_string(), val.clone());
			val
		} else {
			trace!("Reusing asset {}", id);
			self.map.borrow().get(id).unwrap().clone()
		}
	}
}

// Texture & mesh loaders
struct MyLoader<'a>
{
	context: &'a rd::Context,
	asset_root_directory: PathBuf,
	meshes: AssetCache<Mesh<'a>>,
	textures: AssetCache<rd::Texture2D>,
	materials: AssetCache<Material>
}

impl<'a> MyLoader<'a>
{
	fn new(context: &'a rd::Context) -> MyLoader<'a>
	{
		MyLoader {
			context: context,
			asset_root_directory: PathBuf::from("assets"),
			meshes: AssetCache::new(),
			textures: AssetCache::new(),
			materials: AssetCache::new()
		}
	}
}

impl<'a> AssetStore for MyLoader<'a>
{
	fn asset_path(&self, asset_id: &str) -> PathBuf
	{
		self.asset_root_directory.join(&Path::new(asset_id))
	}
}

impl<'a> AssetLoader<Mesh<'a>> for MyLoader<'a>
{
	fn load(&self, asset_id: &str) -> Rc<Mesh<'a>>
	{
		self.meshes.load_with(asset_id, |id| {
			Mesh::load_from_obj(self.context, &self.asset_path(asset_id))
		})
	}
}

impl<'a> AssetLoader<rd::Texture2D> for MyLoader<'a>
{
	fn load(&self, asset_id: &str) -> Rc<rd::Texture2D>
	{
		self.textures.load_with(asset_id, |id| {
			let img = image::open(&self.asset_path(asset_id)).unwrap();
			let (dimx, dimy) = img.dimensions();
			let img2 = img.as_rgb8().unwrap();
			rd::Texture2D::with_pixels(dimx, dimy, 1, rd::TextureFormat::Unorm8x3, Some(img2))
		})
	}
}

impl<'a> AssetLoader<Material> for MyLoader<'a>
{
	fn load(&self, asset_id: &str) -> Rc<Material>
	{
		self.materials.load_with(asset_id, |id| {Material::new(&self.asset_path(asset_id))})
	}
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

	let ctx = rd::Context::new();

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

	let loader = MyLoader::new(&ctx);
	let graphics = Graphics::new(&ctx, &loader);
	let terrain_renderer = TerrainRenderer::new();
	// load sample scene
	let mut scene = Scene::load(&ctx, &loader, &Path::new("assets/scenes/scene.json"));
	let mut offset = (0.0, 0.0);

	win.event_loop(&mut glfw, |event, window| {

		ctx.event(&event);
		camera_controller.event(&event);

		match event {
			Event::Render(dt) => {
				// update camera
				let (vp_width, vp_height) = window.get_size();
				let cam = camera_controller.get_camera(window);

				{
					//let frame = ctx.create_frame(render_target::RenderTarget::Screen);
					//let mut frame = ctx.create_frame(RenderTarget::render_to_texture(vec![&mut tex]));
					let mut frame = ctx.create_frame(rd::RenderTarget::screen((1024, 768)));
					frame.clear(Some([1.0, 0.0, 0.0, 0.0]), Some(1.0));
					//let shader_params = ShaderParams { u_color: Vec3::new(0.0f32, 1.0f32, 0.0f32) };

					//terrain.render_terrain(&terrain, );
					scene.render(&graphics, &terrain_renderer, &cam, &ctx);

					/*{
						use num::traits::One;
						let scene_data_buf = frame.make_uniform_buffer(&scene_data);
						let param_buf_3 = frame.make_uniform_buffer(&shader_params);

						let transform = Mat4::<f32>::one();
						let banana_transform = Iso3::<f32>::one().append_translation(&Vec3::new(offset.0 as f32, offset.1 as f32, 0.0)).to_homogeneous();

						mesh_renderer.draw_mesh(&cube_mesh, &scene_data, &material, &transform, &frame);
						mesh_renderer.draw_mesh(&banana_mesh, &scene_data, &material, &banana_transform, &frame);
					}*/
				}
			},

			// test: move banana
			Event::KeyDown(glfw::Key::Z) => {
				offset = (offset.0, offset.1 + 0.1);
			},

			Event::KeyDown(glfw::Key::S) => {
				offset = (offset.0 + 0.1, offset.1);
			}

			_ => {}
		};

		true
	});

}
