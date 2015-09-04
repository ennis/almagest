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

//-------------------------------------------
// JSON scene representation
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
	entities: Vec<JsonSceneEntity>
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
	mesh: Mesh<'a>,
	material: Material,
	transform: MyTransform
}

pub struct Scene<'a>
{
	entities: Vec<Entity<'a>>	
}

impl<'a> Scene<'a>
{
	/// Load a scene from a JSON file
	/// root: asset folder root
	/// scene: subpath of scene in asset root
	pub fn from_file<'b>(context: &'b Context, root: &Path, scene: &Path) -> Scene<'b>
	{
		use std::io::Read;
		let f = File::open(root.join(scene)).unwrap();
		let reader = BufReader::new(&f);
		// load JSON repr
		let scene_json : JsonSceneFile = serde_json::de::from_reader(reader).unwrap();
		println!("{:?}", scene_json);
		
		// load all meshes and materials
		let mut entities = Vec::<Entity<'b>>::new();
		for scene_ent in scene_json.entities.iter()
		{
			entities.push(Entity {
				mesh: Mesh::load_from_obj(context, &root.join(&Path::new(&scene_ent.mesh))),
				material: Material::new(&root.join(&Path::new(&scene_ent.material))),
				transform: MyTransform {
					position: Vec3::new(scene_ent.transform.position.x, scene_ent.transform.position.y, scene_ent.transform.position.z),
					rotation: Vec3::new(scene_ent.transform.rotation.x, scene_ent.transform.rotation.y, scene_ent.transform.rotation.z),
					scale: scene_ent.transform.scale
				}
			});
		}		
		
		Scene { entities: entities }
	}
	
	pub fn render(&self, mesh_renderer: &MeshRenderer, scene_data: &SceneData, frame: &Frame)
	{
		use num::traits::One;
		for ent in self.entities.iter()
		{
			mesh_renderer.draw_mesh(&ent.mesh, scene_data, &ent.material, &ent.transform.to_mat4(), frame); 
		}
	}
}