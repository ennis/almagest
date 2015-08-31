use gl;
use gl::types::*;
use std::mem;
use typed_arena::{Arena};
use std::cell::RefCell;
use attrib::*;
use shader::{Program};
use draw::*;
use texture::{Texture2D};

pub enum RenderTarget<'a>
{
	Screen,
	TextureTarget {color_target: &'a mut Texture2D, depth_target: &'a mut Texture2D }
}

