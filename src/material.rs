use image;
use image::{GenericImage};
use rendering::*;
use rendering::shader::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::{RefCell};


/// Describes the appearance of an object
pub struct Material
{
	pub main_tex: Rc<Texture2D>,
    pub shader: Rc<Shader>,
}

impl Material
{
	/// create a new material from a shader
	pub fn new_with_shader(shader: Rc<Shader>, main_tex: Rc<Texture2D>) -> Material
	{
		Material
		{
            shader: shader,
			main_tex: main_tex
		}
	}

	/// bind the material to pipeline
	/// currently, the only state set is the texture unit 0
	pub fn bind(&self)
	{
		self.main_tex.bind(0);
	}
}
