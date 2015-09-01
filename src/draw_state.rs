use gl;
use gl::types::*;

#[derive(Copy, Clone, Debug)]
pub enum CullMode 
{
	None, Front, Back, FrontAndBack
}

impl CullMode
{
	fn to_gl(self) -> GLenum
	{
		match self 
		{
			CullMode::None => panic!("CullMode::None"),
			CullMode::Front => gl::FRONT,
			CullMode::Back => gl::BACK,
			CullMode::FrontAndBack => gl::FRONT_AND_BACK
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum PolygonFillMode
{
	Fill, Wireframe
}

impl PolygonFillMode
{
	fn to_gl(self) -> GLenum
	{
		match self 
		{
			PolygonFillMode::Fill => gl::FILL,
			PolygonFillMode::Wireframe => gl::LINE
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum BlendOp
{
	Add,
	Subtract,
	ReverseSubtract,
	Min, 
	Max,
}

#[derive(Copy, Clone, Debug)]
pub enum BlendFactor
{
	Zero,
	One,
	SrcRgb,
	InvSrcRgb,
	DestRgb,
	InvDestRgb,
	SrcAlpha,
	InvSrcAlpha,
	DestAlpha,
	InvDestAlpha
}

#[derive(Copy, Clone, Debug)]
pub struct DrawState
{
	cull_mode: CullMode,
	polygon_fill_mode: PolygonFillMode,
	depth_clip_enable: bool,
	// depth-stencil state
	depth_test_enable: bool,
	depth_write_enable: bool
}

impl DrawState
{
	pub fn default() -> DrawState
	{
		DrawState {
			cull_mode: CullMode::None,
			polygon_fill_mode: PolygonFillMode::Fill,
			depth_clip_enable: true,
			depth_test_enable: true,
			depth_write_enable: true
		}
	}
	
	pub fn default_wireframe() -> DrawState
	{
		DrawState {
			polygon_fill_mode: PolygonFillMode::Wireframe,
			.. DrawState::default()
		}
	}
	
	// TODO state cache, redundant state call elimination
	pub fn sync_state(&self)
	{
		unsafe {
			// set GL states
			if self.depth_test_enable {
				gl::Enable(gl::DEPTH_TEST);
			} else {
				gl::Disable(gl::DEPTH_TEST);
			}
			
			gl::DepthMask(if self.depth_write_enable { gl::TRUE } else { gl::FALSE });
			// TODO? fill mode per face
			gl::PolygonMode(gl::FRONT_AND_BACK, self.polygon_fill_mode.to_gl());
			
			match self.cull_mode 
			{ 
				CullMode::None => gl::Disable(gl::CULL_FACE),
				_ => {				
					gl::Enable(gl::CULL_FACE);
					gl::CullFace(self.cull_mode.to_gl());
				}
			}
			// TODO specify this
			gl::Disable(gl::STENCIL_TEST);
			gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
			gl::DepthFunc(gl::LEQUAL);
		}
	}
}

