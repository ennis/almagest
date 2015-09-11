use material::*;
use std::path::Path;
use serde;
use serde_json;
use nalgebra::*;
use rendering::*;
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
	position: JsonSceneVec3,
	color: JsonSceneColor,
	intensity: f32
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
pub struct JsonSceneEntity
{
	mesh: String,
	material: String,
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

pub struct Entity<'a>
{
	mesh: Rc<Mesh<'a>>,	// mesh index
	material: Rc<Material>,	// material index
	transform: MyTransform
}

pub struct LightSource
{
	position: Vec3<f32>,
	intensity: f32,
	color: Vec3<f32>
}

pub struct Scene<'a>
{
	entities: Vec<Entity<'a>>,
	light_sources: Vec<LightSource>,
	terrain: Option<Terrain<'a>>,
	// Shadow map render target
	shadow_map: Texture2D,
	// shadow render program
	shadow_prog: Program
}

impl<'a> Scene<'a>
{
	/// Load a scene from a JSON file
	/// root: asset folder root
	/// scene: subpath of scene in asset root
	pub fn load<'b, R>(context: &'b Context, loader: &R, scene: &Path) -> Scene<'b>
			where R: AssetStore + AssetLoader<Material> + AssetLoader<Mesh<'b>>
	{
		use std::io::Read;
		let f = File::open(scene).unwrap();
		let reader = BufReader::new(&f);
		// load JSON repr
		let scene_json : JsonSceneFile = serde_json::de::from_reader(reader).unwrap();
		println!("{:?}", scene_json);

		// load all meshes and materials
		let mut entities = Vec::<Entity>::new();
		let mut light_sources = Vec::<LightSource>::new();

		for scene_ent in scene_json.entities.iter()
		{
			entities.push(Entity {
				mesh: loader.load(&scene_ent.mesh),
				material: loader.load(&scene_ent.material),
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
			light_sources.push(LightSource {
				position: Vec3::new(ls.position.x, ls.position.y, ls.position.z),
				intensity: ls.intensity,
				color: Vec3::new(ls.color.r, ls.color.g, ls.color.b)
			});
		}

		// create terrain
		let terrain = scene_json.terrain.map(|t| Terrain::new(context, &loader.asset_path(&t.heightmap), t.scale, t.height_scale));

		Scene {
			entities: entities,
			light_sources: light_sources,
			terrain: terrain,
			shadow_map: Texture2D::new(1024, 1024, 1, TextureFormat::Depth24),
			shadow_prog: Program::from_source(
				&load_shader_source(&loader.asset_path("shaders/default_shadowmap.vs")),
				&load_shader_source(&loader.asset_path("shaders/default_shadowmap.fs"))).expect("Error creating program"),
		}
	}

	pub fn render(&mut self, graphics: &Graphics, terrain_renderer: &TerrainRenderer, cam: &Camera, context: &Context)
	{
		use num::traits::One;

		let light_direction = Vec3::new(0.0f32, -1.0f32, 0.0f32);
		// light matrix setup
		let depth_proj_matrix = OrthoMat3::<f32>::new(20.0, 20.0, -10.0, 10.0);
		let mut depth_view_matrix = Iso3::<f32>::one();
		depth_view_matrix.look_at_z(
			&Pnt3::new(1.0, 1.0, 0.0),
			&Pnt3::new(0.0, 0.0, 0.0),
			&Vec3::new(0.0, 0.0, -1.0));
		depth_view_matrix.inv_mut();

		let light_data = LightData {
			light_dir: light_direction,
			light_matrix: *depth_proj_matrix.as_mat() * depth_view_matrix.to_homogeneous()
		};

		{
			// shadow map: create render target with only one depth map
			let mut shadow_frame = graphics.context().create_frame(RenderTarget::render_to_texture(
				(1024, 1024), vec![], Some(&mut self.shadow_map)));
			//let mut shadow_frame = context.create_frame(RenderTarget::screen((640, 480)));
			shadow_frame.clear(None, Some(1.0));
			for ent in self.entities.iter()
			{
				graphics.draw_mesh_shadow(&ent.mesh, &light_data, &ent.transform.to_mat4(), &shadow_frame);
			}
		}

		{
			// frame for main pass
			let mut frame = graphics.context().create_frame(RenderTarget::screen((1024, 768)));
			frame.clear(Some([0.1, 0.1, 0.2, 1.0]), Some(1.0));
			let rt_dim = frame.dimensions();

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

			// test blit
			//graphics.blit(graphics.default_texture(), &Rect::from_dimensions(0.0, 0.0, 511.0, 511.0), &frame);

			let scene_data = {
				let data = SceneContext {
					view_mat: cam.view_matrix,
					proj_mat: cam.proj_matrix,
					view_proj_mat: cam.proj_matrix * cam.view_matrix,
					light_dir: Vec4::new(1.0,1.0,0.0,0.0),
					w_eye: Vec4::new(0.0,0.0,0.0,0.0),
					viewport_size: Vec2::new(rt_dim.0 as f32, rt_dim.1 as f32),
					light_pos: self.light_sources[0].position,
					light_color: self.light_sources[0].color,
					light_intensity: self.light_sources[0].intensity
				};
				let buf = frame.make_uniform_buffer(&data);
				SceneData {
					data: data,
					buffer: buf.as_raw()
				}
			};

			let light_data = frame.make_uniform_buffer(&depth_matrix);

			// debug shadow map
			graphics.blit(
				&self.shadow_map,
				&Rect { top: 0.0, bottom: 300.0, left: 0.0, right: 300.0 },
				&frame);

			if let Some(ref terrain) = self.terrain
			{
				terrain_renderer.render_terrain(&terrain, &scene_data, &frame);
			}

			for ent in self.entities.iter()
			{
				let model_data = frame.make_uniform_buffer(&ent.transform.to_mat4());
				ent.material.bind();
				self.shadow_map.bind(1);
				graphics.draw_mesh_with_shader(&ent.mesh, &self.shadow_prog,
					&[Binding {slot:0, slice: scene_data.buffer},
					  Binding {slot:1, slice:model_data.as_raw()},
					  Binding {slot:2, slice:light_data.as_raw()}], &frame);
			}
		}
	}
}
