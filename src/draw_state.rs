use gl;
use gl::types::*;

#[derive(Copy, Clone, Debug)]
pub enum CullMode 
{
	None, Front, Back, FrontAndBack
}

#[derive(Copy, Clone, Debug)]
pub enum PolygonFillMode
{
	Fill, Wireframe
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
}

