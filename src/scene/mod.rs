use camera::Camera;

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

// Axis-aligned bounding boxes
pub struct BoundingBox 
{
	xlow  : f32,
	xhigh : f32,
	ylow  : f32,
	yhigh : f32,
	zlow  : f32,
	zhigh : f32
}

pub struct Entity
{
	mesh: Rc<Mesh>,
	material: Rc<Material>,
	transform: MyTransform,
	local_bounds: BoundingBox
}

pub enum LightSource
{
	// direction, color, intensity
	Directional(Vec3<f32>, Vec3<f32>, f32),
	// position, color, intensity
	Point(Vec3<f32>, Vec3<f32>, f32)
}

/// A scene loaded from a JSON description file or created manually
pub struct Scene
{
	pub entities: Vec<Entity>,
	pub light_sources: Vec<LightSource>,
	pub initial_camera: Camera
}

pub impl Scene
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
				},
				local_bounds: /*TODO*/
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

		// display shaders

		Scene {
			entities: entities,
			light_sources: light_sources,
			initial_camera: /* TODO */
		}
	}
}