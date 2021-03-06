use nalgebra::*;
use rendering::RawBufSlice;

pub struct SceneData<'a>
{
	/// scene context
	pub data: SceneContext,
	/// GPU buffer slice containing the scene context
	pub buffer: RawBufSlice<'a>
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SceneContext
{
	pub view_mat: Mat4<f32>,
	pub proj_mat: Mat4<f32>,
	pub view_proj_mat: Mat4<f32>,
	pub light_dir: Vec4<f32>,
	pub w_eye: Vec4<f32>,
	pub viewport_size: Vec2<f32>,
	pub light_pos: Vec3<f32>,
	pub light_color: Vec3<f32>,
	pub light_intensity: f32
}
