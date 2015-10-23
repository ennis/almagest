use material::*;
use std::path::{Path, PathBuf};
use serde;
use serde_json;
use nalgebra::*;
use rendering::*;
use rendering::shader::*;
use std::fs::{File};
use std::io::{BufReader};
use scene_data::*;
use camera::*;
use std::collections::{HashMap};
use asset_loader::*;
use std::rc::Rc;
use terrain::{Terrain, TerrainRenderer};
use shadow_pass::*;
use graphics::*;
use image::{self, GenericImage};
use event::*;
use player::*;
use glfw;
use window::*;

//-------------------------------------------
// JSON scene representation
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSceneTerrain
{
	heightmap: String,
	scale: f32,
	height_scale: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSceneColor
{
	r: f32,
	g: f32,
	b: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSceneLightSource
{
	transform: JsonSceneTransform,
	color: JsonSceneColor,
	intensity: f32,
	mode: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSceneVec3
{
	x: f32,
	y: f32,
	z: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSceneTransform
{
	position: JsonSceneVec3,
	rotation: JsonSceneVec3,
	scale: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSceneMaterial
{
	shader: Option<String>,
	texture: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSceneEntity
{
	mesh: String,
	material: JsonSceneMaterial,
	transform: JsonSceneTransform
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSceneFile
{
	light_sources: Vec<JsonSceneLightSource>,
	entities: Vec<JsonSceneEntity>,
	terrain: Option<JsonSceneTerrain>
}
// end JSON repr
//-------------------------------------------

pub struct MyTransform
{
	position: Vec3<f32>,
	rotation: Vec3<f32>,
	scale: f32
}

impl MyTransform
{
	pub fn to_mat4(&self) -> Mat4<f32>
	{
		use num::traits::One;
		Iso3::<f32>::one()
			.append_translation(&self.position)
			.append_rotation(&self.rotation).to_homogeneous()
	}
}

pub struct Entity
{
	mesh: Rc<Mesh>,	// mesh index
	material: Rc<Material>,	// material index
	transform: MyTransform
}

pub enum LightSource
{
	// direction, color, intensity
	Directional(Vec3<f32>, Vec3<f32>, f32),
	// position, color, intensity
	Point(Vec3<f32>, Vec3<f32>, f32)
}

struct SkyDomeVertex
{
	pos: Vec3<f32>
}

struct Sky
{
	dome_mesh: Mesh,
	shader: Shader,
	pso: PipelineState,
	nightsky: Texture2D
}

#[derive(Copy, Clone, Debug)]
enum DisplayMode 
{
	Shade,
	Shadow,
	Normals,
	Depth
}

pub struct Scene
{
	entities: Vec<Entity>,
	light_sources: Vec<LightSource>,
	terrain: Option<Terrain>,
	// Shadow map render target
	shadow_map: Texture2D,
	//
	shader_cache: ShaderCache,
	depth_only_pso: PipelineState,
	normals_only_pso: PipelineState,
	player_cam: PlayerCamera,
	sky: Sky,
	mode: DisplayMode,
	mode_index: usize
}

fn make_scale_matrix(scale: f32) -> Mat4<f32>
{
	Mat4::new(
		scale, 0.0, 0.0, 1.0,
		0.0, scale, 0.0, 1.0,
		0.0, 0.0, scale, 1.0,
		0.0, 0.0, 0.0, 1.0
		)
}


/// Make a PSO directly from a shader file
/// Using the default draw states
pub fn load_pipeline_state(path: &Path, kw: Keywords) -> PipelineState
{
	let shader = Shader::load(path);
	shader.make_pipeline_state(&PipelineStateDesc {
		keywords: kw,
        pass: StdPass::ForwardBase,
        default_draw_state: DrawState::default(),
        sampler_block_base: 0,
        uniform_block_base: 0
	})
}

#[derive(Copy,Clone)]
#[repr(C)]
struct SkyParams
{
	model_matrix: Mat4<f32>,
	rayleigh_coefficient: f32,
	mie_coefficient: f32,
	mie_directional_g: f32,
	turbidity: f32
}

impl Scene
{
	/// Load a scene from a JSON file
	/// root: asset folder root
	/// scene: subpath of scene in asset root
	pub fn load(context: &Context, asset_root: &Path, scene: &Path) -> Scene
	{
		use std::io::Read;
		let f = File::open(scene).unwrap();
		let reader = BufReader::new(&f);
		// load JSON repr
		let scene_json : JsonSceneFile = serde_json::de::from_reader(reader).unwrap();
		//trace!("{:?}", scene_json);

		let sky_dome = Mesh::load_from_obj(context, &asset_root.join("models/dome.obj"));
		let nightsky = {
			let img = image::open(&asset_root.join("img/skymap.tif")).unwrap();
			let (dimx, dimy) = img.dimensions();
			let img2 = img.as_rgb8().unwrap();
			Texture2D::with_pixels(dimx, dimy, 1, TextureFormat::Unorm8x3, Some(img2))
		};
		let sky_shader = Shader::load(&asset_root.join("shaders/sky.glsl"));
		let sky_pso = sky_shader.make_pipeline_state(&PipelineStateDesc {
            keywords: Keywords::empty(),
            pass: StdPass::ForwardBase,
            default_draw_state: DrawState::default(),
            sampler_block_base: 0,
            uniform_block_base: 0
        });

		// load all meshes and materials
		let mut entities = Vec::<Entity>::new();
		let mut light_sources = Vec::<LightSource>::new();

		let materials = AssetCache::<Material>::new();
		let meshes = AssetCache::<Mesh>::new();
		let textures = AssetCache::<Texture2D>::new();
		let shaders = AssetCache::<Shader>::new();

		for scene_ent in scene_json.entities.iter()
		{
			//info!("*** Loading entity {:?} ***", scene_ent);

			let texture_path = if let Some(ref path) = scene_ent.material.texture {
					&(*path)[..]
				} else {
					"img/missing_512.png"
				};

			let texture = textures.load_with(texture_path,
					&|path| {
						let img = image::open(&asset_root.join(path)).unwrap();
						let (dimx, dimy) = img.dimensions();
						// TODO correctly handle different formats
						let img2 = img.as_rgb8().unwrap();
						Texture2D::with_pixels(dimx, dimy, 1, TextureFormat::Unorm8x3, Some(img2))
					});

			let shader_name = if let Some(ref s) = scene_ent.material.shader {
					asset_root.join(s)
				} else {
					asset_root.join("shaders/default.glsl")
				};

			let shader = shaders.load_with(shader_name.to_str().unwrap(), &|_| {
				Shader::load(&shader_name)
			});


			let material = Rc::new(Material::new_with_shader(
				shader,
				texture));

			entities.push(Entity {
				mesh: meshes.load_with(&scene_ent.mesh, &|path| {
					Mesh::load_from_obj(context, &asset_root.join(path))
				}),
				material: material,
				transform: MyTransform {
					position: Vec3::new(scene_ent.transform.position.x, scene_ent.transform.position.y, scene_ent.transform.position.z),
					rotation: Vec3::new(scene_ent.transform.rotation.x, scene_ent.transform.rotation.y, scene_ent.transform.rotation.z),
					scale: scene_ent.transform.scale
				}
			});
		}

		// setup light sources
		for ls in scene_json.light_sources.iter()
		{
			if ls.mode == "directional"
			{
				light_sources.push(LightSource::Directional(
					Vec3::new(ls.transform.rotation.x, ls.transform.rotation.y, ls.transform.rotation.z),
					Vec3::new(ls.color.r, ls.color.g, ls.color.b),
					ls.intensity));

			}
			else if ls.mode == "point"
			{
				light_sources.push(LightSource::Point(
					Vec3::new(ls.transform.position.x, ls.transform.position.y, ls.transform.position.z),
					Vec3::new(ls.color.r, ls.color.g, ls.color.b),
					ls.intensity));
			}
		}

		// create terrain
		let terrain = scene_json.terrain.map(|t| Terrain::new(context, &asset_root.join(&t.heightmap), t.scale, t.height_scale));

		// display shaders

		Scene {
			sky: Sky {
				dome_mesh: sky_dome,
				nightsky: nightsky,
				shader: sky_shader,
				pso: sky_pso
			},
			mode: DisplayMode::Shade,
			mode_index: 0,
			entities: entities,
			light_sources: light_sources,
			terrain: terrain,
			depth_only_pso: load_pipeline_state(&asset_root.join("shaders/render_depth.glsl"), Keywords::empty()),
			normals_only_pso: load_pipeline_state(&asset_root.join("shaders/render_normals.glsl"), Keywords::empty()),
			shadow_map: Texture2D::new(1024, 1024, 1, TextureFormat::Depth24),
			shader_cache: ShaderCache::new(),
			player_cam: PlayerCamera::new(PlayerCameraSettings
				{
				    field_of_view: 45.0,
					near_plane: 0.01,
					far_plane: 1000.0,
					sensitivity: 0.01
				})
		}
	}

	pub fn event(&mut self, event: &Event)
	{
		self.player_cam.event(event);

		const DISPLAY_MODE_CYCLE : [DisplayMode; 4] = [
			DisplayMode::Shade, 
			DisplayMode::Normals, 
			DisplayMode::Shadow, 
			DisplayMode::Depth];
		match event 
		{
			&Event::KeyDown(glfw::Key::F) => {
				// cycle display modes
				self.mode_index += 1;
				self.mode_index %= DISPLAY_MODE_CYCLE.len();
				self.mode = DISPLAY_MODE_CYCLE[self.mode_index];
				println!("DisplayMode: {:?}", self.mode);
			},
			_ => {}
		}
	}

	pub fn update(&mut self, dt: f64, input: &Input)
	{
		self.player_cam.update(dt, input);
	}

	pub fn render(&mut self, graphics: &Graphics, terrain_renderer: &TerrainRenderer, window: &Window, context: &Context, cam: &Camera)
	{
		use num::traits::One;

		let (width, height) = window.dimensions();

		// XXX these should be constants
		let pass_cfg_shadow = PipelineStateDesc {
				keywords: Keywords::empty(),
				pass: StdPass::Shadow,
				default_draw_state: DrawState::default(),
				sampler_block_base: 0,
				uniform_block_base: 2
		};

		let pass_cfg_forward = PipelineStateDesc {
				keywords: POINT_LIGHT | SHADOWS_SIMPLE,
				pass: StdPass::ForwardBase,
				default_draw_state: DrawState::default(),
				sampler_block_base: 0,
				uniform_block_base: 2
		};

		// TODO use another camera if there is no terrain
		//let cam = self.player_cam.get_camera(&self.terrain.as_ref().unwrap(), window);

		// TODO do not assume that the first light source is directional
		let light_direction = if let LightSource::Directional(dir, _, _) = self.light_sources[0] {
			dir
		} else {
			Vec3::new(0.0f32, -1.0f32, 0.0f32)
		};

		let (light_color, light_intensity) = match self.light_sources[0] {
			LightSource::Directional(_, color, intensity) => (color, intensity),
			LightSource::Point(_, color, intensity) => (color, intensity)
		};

		// light matrix setup
		// TODO compute bounding box of view frustum?
		// TODO cascaded shadow maps
		let depth_proj_matrix = OrthoMat3::<f32>::new(20.0, 20.0, -10.0, 10.0);
		let mut depth_view_matrix = Iso3::<f32>::one();
		depth_view_matrix.look_at_z(
			&Pnt3::new(0.0, 0.0, 0.0),
			&(Pnt3::new(0.0, 0.0, 0.0) + light_direction),
			&Vec3::new(0.0, 0.0, -1.0));
		depth_view_matrix.inv_mut();

		let light_data = LightData {
			light_dir: light_direction,
			light_matrix: *depth_proj_matrix.as_mat() * depth_view_matrix.to_homogeneous()
		};

		{
			// shadow map: create render target with only one depth map
			let mut shadow_frame = graphics.context().create_frame(
				&[], Some(self.shadow_map.view_as_depth_stencil_target()));

			//let mut shadow_frame = context.create_frame(RenderTarget::screen((640, 480)));
			shadow_frame.clear(None, Some(1.0));
			for ent in self.entities.iter()
			{
				#[repr(C)]
				#[derive(Copy, Clone)]
				struct LightParams
				{
					light_matrix: Mat4<f32>,
					model_matrix: Mat4<f32>
				}

				let model_data = shadow_frame.make_uniform_buffer(&ent.transform.to_mat4());

				let light_params = shadow_frame.make_uniform_buffer(&LightParams {
					light_matrix: light_data.light_matrix,
					model_matrix: ent.transform.to_mat4()
					});

				graphics.draw_mesh_with_shader(
					&ent.mesh,
					&self.shader_cache.get(&ent.material.shader, &pass_cfg_shadow),
					&[Binding{slot:0, slice: light_params.as_raw()},
					  Binding{slot:1, slice: model_data.as_raw()}],
					&shadow_frame);
			}
		}

		{
			// frame for main pass
			let mut frame = graphics.context().create_screen_frame(window);
			frame.clear(Some([0.1, 0.1, 0.2, 1.0]), Some(1.0));
			let rt_dim = frame.dimensions();

			// For shadows:
			// depth matrix with bias
			let depth_matrix = {
				let bias = Mat4::<f32>::new(
					0.5, 0.0, 0.0, 0.5,
					0.0, 0.5, 0.0, 0.5,
					0.0, 0.0, 0.5, 0.5,
					0.0, 0.0, 0.0, 1.0
					);
				bias * light_data.light_matrix
			};

			let scene_data = {
				let data = SceneContext {
					view_mat: cam.view_matrix,
					proj_mat: cam.proj_matrix,
					view_proj_mat: cam.proj_matrix * cam.view_matrix,
					light_dir: Vec4::new(
						light_direction.x,
						-light_direction.y,
						light_direction.z,
						0.0),
					w_eye: Vec4::new(cam.w_eye.x,cam.w_eye.y,cam.w_eye.z,1.0),
					viewport_size: Vec2::new(rt_dim.0 as f32, rt_dim.1 as f32),
					light_pos: light_direction,
					light_color: light_color,
					light_intensity: light_intensity
				};
				let buf = frame.make_uniform_buffer(&data);
				SceneData {
					data: data,
					buffer: buf.as_raw()
				}
			};

			let light_data = frame.make_uniform_buffer(&depth_matrix);

			// debug shadow map
			/*graphics.blit(
				&self.shadow_map,
				&Rect { top: 0.0, bottom: 300.0, left: 0.0, right: 300.0 },
				&frame);*/


			match self.mode 
			{
				//================================================
				//
				// Render the full scene with shading
				//
				DisplayMode::Shade => {
					//================================================
					// TERRAIN
					if let Some(ref terrain) = self.terrain
					{
						terrain_renderer.render_terrain(&terrain, &scene_data, &frame);
					}

					//================================================
					// SKY
					{
						let model_data = frame.make_uniform_buffer(
							&SkyParams {
								model_matrix: make_scale_matrix(100.0),
								rayleigh_coefficient: 0.0,	// unused
								mie_coefficient: 0.005,
								mie_directional_g: 0.80,
								turbidity: 5.0
							});

						self.sky.nightsky.bind(0);	// TODO fix this hack
						graphics.draw_mesh_with_shader(
							&self.sky.dome_mesh,
							&self.sky.pso,
							&[Binding {slot:0, slice:scene_data.buffer},
							  Binding {slot:1, slice:model_data.as_raw()},
							  Binding {slot:2, slice:light_data.as_raw()}],
							&frame);
					}

					//================================================
					// SCENE
					for ent in self.entities.iter()
					{
						let model_data = frame.make_uniform_buffer(&ent.transform.to_mat4());
						ent.material.bind();
						self.shadow_map.bind(1);
						graphics.draw_mesh_with_shader(
							&ent.mesh,
							&self.shader_cache.get(&ent.material.shader, &pass_cfg_forward),
							&[Binding {slot:0, slice:scene_data.buffer},
							  Binding {slot:1, slice:model_data.as_raw()},
							  Binding {slot:2, slice:light_data.as_raw()}], &frame);
					}
				},

				//================================================
				//
				// Render only scene items, without shadows,
				// sky and terrain
				DisplayMode::Normals => {
					//================================================
					// SCENE
					for ent in self.entities.iter()
					{
						let model_data = frame.make_uniform_buffer(&ent.transform.to_mat4());
						graphics.draw_mesh_with_shader(
							&ent.mesh,
							&self.normals_only_pso,
							&[Binding {slot:0, slice:scene_data.buffer},
							  Binding {slot:1, slice:model_data.as_raw()}], 
							&frame);
					}
				},
				//================================================
				//
				// Render and show depth
				//
				DisplayMode::Depth => {
					//================================================
					// SCENE
					for ent in self.entities.iter()
					{
						let model_data = frame.make_uniform_buffer(&ent.transform.to_mat4());
						graphics.draw_mesh_with_shader(
							&ent.mesh,
							&self.depth_only_pso,
							&[Binding {slot:0, slice:scene_data.buffer},
							  Binding {slot:1, slice:model_data.as_raw()}], 
							&frame);
					}
				},
				//================================================
				//
				// Render and show shadow maps
				//
				DisplayMode::Shadow => {
					// unimplemented
				}
			}


		}
	}
}
