use gl;
use gl::types::*;
use std::mem;
use rendering::buffer::{RawBuffer, BufSlice, RawBufSlice, BufferAccess,
			BufferBindingHint, BufferUsage,
			BufferAllocator, as_byte_slice, object_as_byte_slice, Binding};
use typed_arena::{Arena};
use std::cell::RefCell;
use rendering::attrib::*;
use rendering::texture::Texture2D;
use rendering::sampler::Sampler2D;
use rendering::shader::*;

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

pub struct TextureBinding<'a>
{
	pub slot: u32,
	pub sampler: &'a Sampler2D,
	pub texture: &'a Texture2D
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

#[derive(Copy, Clone)]
pub enum PrimitiveType
{
	Point,
	Line,
	Triangle
}

impl PrimitiveType
{
	pub fn to_gl_mode(self) -> u32
	{
		match self {
			PrimitiveType::Point => gl::POINTS,
			PrimitiveType::Line => gl::LINES,
			PrimitiveType::Triangle => gl::TRIANGLES
		}
	}
}

#[derive(Copy, Clone)]
pub struct MeshPart
{
	pub primitive_type: PrimitiveType,
	pub start_vertex: u32,
	pub start_index: u32,
	pub num_vertices: u32,
	pub num_indices: u32
}


pub fn draw_instanced(
		vertex_buffer: RawBufSlice,
		index_buffer: Option<RawBufSlice>,
		part: MeshPart,
		shader: &Shader,
		pipeline_state: &PipelineState,
		uniform_buffers: &[Binding],
		textures: &[TextureBinding])
{
	pipeline_state.draw_state.sync_state();
	unsafe
	{
		gl::UseProgram(pipeline_state.program.obj);
		super::buffer::bind_uniform_buffers(uniform_buffers);
		super::buffer::bind_vertex_buffers(&shader.layout, &[vertex_buffer]);

		for t in textures.iter() {
			t.texture.bind(t.slot as u32);
			t.sampler.bind(t.slot as u32);
		}

		if let Some(ref ib) = index_buffer {
			ib.raw.bind_as_element_array();
			gl::DrawElementsInstancedBaseVertexBaseInstance(
				part.primitive_type.to_gl_mode(),
				part.num_indices as i32,
				gl::UNSIGNED_SHORT,
				(part.start_index * 2) as *const GLvoid,
				1, part.start_vertex as i32, 0);
		}
		else
		{
			gl::DrawArraysInstanced(
				part.primitive_type.to_gl_mode(),
				part.start_vertex as i32,
				part.num_vertices as i32,
				1);
		}
	}
}

pub struct RenderTarget<'a>
{
	viewport: (i32, i32, i32, i32),
	output: RenderTargetOutput<'a>
}

pub enum RenderTargetOutput<'a>
{
	Screen,
	Texture { color_targets: Vec<&'a mut Texture2D>, depth_target: Option<&'a mut Texture2D> }
}

impl<'a> RenderTarget<'a>
{
	pub fn screen(screen_size: (i32, i32)) -> RenderTarget<'a>
	{
		RenderTarget {
			viewport: (0, 0, screen_size.0, screen_size.1),
			output: RenderTargetOutput::Screen
		}
	}

	pub fn render_to_texture(
		viewport_size: (i32, i32),
		color_targets: Vec<&'a mut Texture2D>,
		depth_target: Option<&'a mut Texture2D> ) -> RenderTarget<'a>
	{
		// TODO check that all color & depth targets have the same size
		RenderTarget {
			viewport: (0, 0, viewport_size.0, viewport_size.1),
			output: RenderTargetOutput::Texture { color_targets: color_targets, depth_target: depth_target }
		}
	}
}

pub struct Frame<'a>
{
	buffer_allocator: &'a BufferAllocator,
	temporary_buffers: Arena<RawBuffer<'a>>,
	render_target: RenderTarget<'a>,
	framebuffer: GLuint
}


fn create_framebuffer(color_targets: &[&mut Texture2D], depth_target: &Option<&mut Texture2D>) -> GLuint
{
	let mut fbo : GLuint = 0;
	unsafe {
		gl::GenFramebuffers(1, &mut fbo);
		gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

		// bind color attachements
		for (i, tex) in color_targets.iter().enumerate() {
			// TODO support targets other than 2d textures
			// (texture layers, cube map faces, whole cube map, etc.)
			gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as u32, tex.obj, 0);
		}

		// working with borrows of &Option<&mut T> are a bit awkward
		if let &Some(ref depth_target) = depth_target
		{
			gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, depth_target.obj, 0);
		}

		let draw_buffers = [
			gl::COLOR_ATTACHMENT0,
			gl::COLOR_ATTACHMENT0 + 1,
			gl::COLOR_ATTACHMENT0 + 2,
			gl::COLOR_ATTACHMENT0 + 3,
			gl::COLOR_ATTACHMENT0 + 4,
			gl::COLOR_ATTACHMENT0 + 5,
			gl::COLOR_ATTACHMENT0 + 6,
			gl::COLOR_ATTACHMENT0 + 7
		];

		gl::DrawBuffers(color_targets.len() as GLsizei, draw_buffers[..].as_ptr());
		assert!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE);
	}

	fbo
}

impl<'a> Frame<'a>
{
	pub fn dimensions(&self) -> (u32, u32)
	{
		(self.render_target.viewport.2 as u32, self.render_target.viewport.3 as u32)
	}

	pub fn new(
		buffer_allocator: &'a BufferAllocator,
		render_target: RenderTarget<'a>) -> Frame<'a>
	{
		let fbo = match render_target.output {
			RenderTargetOutput::Screen => 0,
			RenderTargetOutput::Texture { ref color_targets, ref depth_target } => create_framebuffer(&color_targets[..], depth_target)
		};

		unsafe {
			gl::Viewport(render_target.viewport.0, render_target.viewport.1, render_target.viewport.2, render_target.viewport.3);
		}

		Frame {
			buffer_allocator: buffer_allocator,
			temporary_buffers: Arena::with_capacity(300),
			render_target: render_target,
			framebuffer: fbo
		}


		// TODO setup default state elsewhere
	}

	pub fn clear(&mut self, color: Option<[f32; 4]>, depth: Option<f32>)
	{
		match (color, depth) {
			(Some(color), Some(depth)) => {
				unsafe {
					gl::ClearColor(color[0], color[1], color[2], color[3]);
					gl::ClearDepth(depth as GLclampd);
					gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
				}
			},
			(Some(color), None) => {
				unsafe {
					gl::ClearColor(color[0], color[1], color[2], color[3]);
					gl::Clear(gl::COLOR_BUFFER_BIT);
				}
			},
			(None, Some(depth)) => {
				unsafe {
					gl::ClearDepth(depth as GLclampd);
					gl::Clear(gl::DEPTH_BUFFER_BIT);
				}
			}
			_ => {}
		}
	}

	pub fn alloc_temporary_buffer<'b, T: Copy>(
		&'b self,
		num_elements: usize,
		binding: BufferBindingHint ,
		initial_data: Option<&[T]>) -> BufSlice<'b, T>
	{
		if let Some(d) = initial_data {
			assert!(num_elements == d.len());
		}
		let buf = self.buffer_allocator.alloc_raw_buffer(
			num_elements * mem::size_of::<T>(),
			BufferAccess::WriteOnly,
			binding,
			BufferUsage::Stream,
			initial_data.map(|d| as_byte_slice(d)));
		unsafe {
			self.temporary_buffers.alloc(buf).as_buf_slice(0, num_elements)
		}
	}

	pub fn make_uniform_buffer<'b, T: Copy>(
		&'b self,
		initial_data: &T) -> BufSlice<'b, T>
	{
		let buf = self.buffer_allocator.alloc_raw_buffer(
			mem::size_of::<T>(),
			BufferAccess::WriteOnly,
			BufferBindingHint::UniformBuffer,
			BufferUsage::Stream,
			Some(object_as_byte_slice(initial_data)));
		unsafe {
			self.temporary_buffers.alloc(buf).as_buf_slice(0, 1)
		}
	}

	// TODO make a 'DrawCallBuilder' with sensible defaults for the arguments
	pub fn draw(
		&self,
		vertex_buffer: RawBufSlice,
		index_buffer: Option<RawBufSlice>,
		shader: &Shader,
		pipeline_state: &PipelineState,
		mesh_part: MeshPart,
		uniform_buffers: &[Binding],
		textures: &[TextureBinding])
	{
		// HERE: rebind framebuffer if necessary
		unsafe {
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
		}
		draw_instanced(vertex_buffer, index_buffer, mesh_part, shader, pipeline_state, uniform_buffers, textures);
	}
}

impl<'a> Drop for Frame<'a>
{
	fn drop(&mut self)
	{
		unsafe {
			gl::DeleteFramebuffers(1, &mut self.framebuffer);
		}
	}
}
