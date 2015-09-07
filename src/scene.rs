use mesh::*;
use material::*;
use std::path::Path;
use serde;
use serde_json;
use nalgebra::*;
use context::*;
use std::fs::{File};
use std::io::{BufReader};
use frame::*;
use scene_data::*;
use camera::*;
use std::collections::{HashMap};
use asset_loader::*;
use std::rc::Rc;
use terrain::{Terrain, TerrainRenderer};

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
	terrain: Option<Terrain<'a>>
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
			terrain: terrain
		}
	}

	pub fn render(&self, mesh_renderer: &MeshRenderer, terrain_renderer: &TerrainRenderer, cam: &Camera, frame: &Frame)
	{
		let rt_dim = frame.dimensions();
		let scene_data = SceneData {
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

		if let Some(ref terrain) = self.terrain
		{
			terrain_renderer.render_terrain(&terrain, &scene_data, &frame);
		}

		for ent in self.entities.iter()
		{
			mesh_renderer.draw_mesh(&ent.mesh, &scene_data, &ent.material, &ent.transform.to_mat4(), frame);
		}
	}
}
