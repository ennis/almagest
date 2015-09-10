use image;
use image::{GenericImage};
use rendering::*;
use std::path::Path;

/// Describes the appearance of an object
pub struct Material
{
	main_tex: Texture2D
}

impl Material
{
	/// create a new material from an image file
	pub fn new(main_tex_path: &Path) -> Material
	{
		let img = image::open(main_tex_path).unwrap();
		let (w, h) = img.dimensions();
		let img2 = img.as_rgb8().unwrap();

		Material {
			main_tex: Texture2D::with_pixels(w, h, 1, TextureFormat::Unorm8x3, Some(img2))
		}
	}

	/// bind the material to pipeline
	/// currently, the only state set is the texture unit 0
	pub fn bind(&self)
	{
		self.main_tex.bind(0);
	}
}
