use mesh::*;
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
	// blitter shader (for debugging)
	blit_shader: Program,
	// dummy layout for blitter
	blitter_layout: InputLayout
}

struct Rect
{
	top: f32,
	bottom: f32,
	left: f32,
	right: f32
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
			blit_shader: Program::from_source(
					&load_shader_source(Path::new("assets/shaders/blit.vs")),
					&load_shader_source(Path::new("assets/shaders/blit.fs"))).expect("Error creating program"),
			blitter_layout: InputLayout::new(1, &[
				Attribute { slot:0, ty: AttributeType::Float2 },
				Attribute { slot:0, ty: AttributeType::Float2 }] )
		}
	}

	pub fn blit_tex(&self, texture: &Texture2D, frame: &Frame, rect: &Rect, scene_data_buf: RawBufSlice)
	{
		texture.bind(0);
		// blit rectangle

		#[derive(Copy, Clone)]
		#[repr(C)]
		struct Vertex2D {
			pos: [f32; 2],
			tex: [f32; 2]
		}
		let buf = frame.alloc_temporary_buffer(6, BufferBindingHint::VertexBuffer, Some(&[
			Vertex2D { pos : [rect.top, rect.left], tex : [0.0, 0.0] },
			Vertex2D { pos : [rect.top, rect.right],  tex: [0.0, 1.0] },
			Vertex2D { pos : [rect.bottom, rect.left], tex: [1.0, 0.0] },
			Vertex2D { pos : [rect.bottom, rect.left], tex: [1.0, 0.0] },
			Vertex2D { pos : [rect.top, rect.right], tex: [0.0, 1.0] },
			Vertex2D { pos : [rect.bottom, rect.right], tex: [1.0, 1.0] }
			]));
		frame.draw(
			buf.as_raw(),
			None,
			&DrawState::default(),
			&self.blitter_layout,
			MeshPart {
				primitive_type: PrimitiveType::Triangle,
				start_vertex: 0,
				start_index: 0,
				num_vertices: 6,
				num_indices: 0
				},
			&self.blit_shader,
			&[Binding {slot:0, slice:scene_data_buf}],
			&[texture]
			);
	}

	pub fn render(&mut self, mesh_renderer: &MeshRenderer, terrain_renderer: &TerrainRenderer, cam: &Camera, context: &Context)
	{
		use num::traits::One;
		{
			// shadow map: create render target with only one depth map
			let mut shadow_frame = context.create_frame(RenderTarget::render_to_texture(
				(1024, 1024), vec![], Some(&mut self.shadow_map)));
			//let mut shadow_frame = context.create_frame(RenderTarget::screen((640, 480)));
			shadow_frame.clear(None, Some(1.0));
			// light direction
			let light_direction = Vec3::new(0.0f32, -1.0f32, 0.0f32);

			// light matrix setup
			let depth_proj_matrix = OrthoMat3::<f32>::new(20.0, 20.0, -10.0, 10.0);
			let mut depth_view_matrix = Iso3::<f32>::one();
			depth_view_matrix.look_at_z(
				&Pnt3::new(0.0, 1.0, 0.0),
				&Pnt3::new(0.0, 0.0, 0.0),
				&Vec3::new(0.0, 0.0, -1.0));
			depth_view_matrix.inv_mut();

			let light_data = LightData {
				light_dir: light_direction,
				light_matrix: *depth_proj_matrix.as_mat() * depth_view_matrix.to_homogeneous()
			};

			for ent in self.entities.iter()
			{
				mesh_renderer.draw_mesh_shadow(&ent.mesh, &light_data, &ent.transform.to_mat4(), &shadow_frame);
			}

		}

		{
			// frame for main pass
			let mut frame = context.create_frame(RenderTarget::screen((640, 480)));
			frame.clear(Some([1.0, 0.0, 0.0, 0.0]), Some(1.0));
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

			// debug shadow map
			let buf = frame.make_uniform_buffer(&scene_data);
			self.blit_tex(&self.shadow_map, &frame, &Rect { top: 0.0, bottom: 100.0, left: 0.0, right: 100.0}, buf.as_raw());

			if let Some(ref terrain) = self.terrain
			{
				terrain_renderer.render_terrain(&terrain, &scene_data, &frame);
			}

			for ent in self.entities.iter()
			{
				mesh_renderer.draw_mesh(&ent.mesh, &scene_data, &ent.material, &ent.transform.to_mat4(), &frame);
			}
		}
	}
}
