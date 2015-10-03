use rendering::buffer::*;
use rendering::texture::*;
use rendering::attrib::*;
use rendering::sampler::Sampler2D;
use rendering::shader::*;
use gl::types::*;
use gl;
use libc::c_void;
use std::ffi::CStr;
use event::*;
use window::Window;
use std::mem;
use typed_arena::Arena;


/// TODO: TextureViews should be typed
pub struct TextureView
{
	pub texture: GLuint
}

#[derive(Copy, Clone)]
pub struct RenderTargetView
{
	pub texture: GLuint,
	pub viewport: (u32, u32, u32, u32)
}

#[derive(Copy, Clone)]
pub struct DepthStencilView
{
	pub texture: GLuint,
	pub viewport: (u32, u32, u32, u32)
}

// TODO the context should know the window dimensions
// (and be notified when the window size changes)
pub struct Context
{
	window_size: (i32, i32)
	// three previous frames
	//last_frames: [Option<Frame<'a>>; 3]
	// TODO ref to window
}

// Views: persistent or scoped?
// Persistent: like D3D, create the view object only once (borrowing the texture, but not mutably)
// *** Scoped: the view object is created only when it is needed, and dropped when it is not needed anymore.
//     for render targets, this allows the object to 'lock' the underlying texture for writing, preventing
//     it to be bound as a texture at the same time (undefined behavior in OpenGL)
//
// D3D11: views = objects
// D3D12: views = descriptors (possibly on GPU memory)
// OpenGL: views defined when creating a framebuffer
// RTVs do not change between frames, why recreate them?
// 		=> RTV update on window resize?
// Pre-create

/*pub enum RenderTargetView<'a>
{
	// TODO: texture layers, buffers
	Texture2D(&'a mut Texture2D),
	Screen,
	None,
}

pub enum DepthStencilView<'a>
{
	Texture2D(&'a mut Texture2D),
	Screen,
	None,
}

pub struct CommandList<'a>
{
	context: &'a Context
}

impl<'a> CommandList<'a>
{
	pub fn alloc_transient_buffer()
	{
		unimplemented!()
	}
}*/

extern "system" fn debug_callback(
	source: GLenum,
	ty: GLenum,
	id: GLuint,
	severity: GLenum,
	length: GLsizei,
	msg: *const GLchar,
	data: *mut c_void)
{
	unsafe {
		let msg_str = CStr::from_ptr(msg);
		//trace!("{}", msg_str.to_str().unwrap());
	}
}


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
	pub cull_mode: CullMode,
	pub polygon_fill_mode: PolygonFillMode,
	pub depth_clip_enable: bool,
	// depth-stencil state
	pub depth_test_enable: bool,
	pub depth_write_enable: bool
}

pub struct TextureBinding<'a>
{
	pub slot: u32,
	pub sampler: &'a Sampler2D,
	pub texture: &'a Texture2D
}

impl DrawState
{
	pub const fn default() -> DrawState
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
		pipeline_state: &PipelineState,
		uniform_buffers: &[Binding],
		textures: &[TextureBinding])
{
	pipeline_state.draw_state.sync_state();
	unsafe
	{
		gl::UseProgram(pipeline_state.program.obj);
		super::buffer::bind_uniform_buffers(uniform_buffers);
		super::buffer::bind_vertex_buffers(&pipeline_state.layout, &[vertex_buffer]);

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

impl Context
{
	pub fn new() -> Context {
		unsafe {
			// enable OpenGL debug output
			gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
			gl::DebugMessageCallback(debug_callback, 0 as *const c_void);
			gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, 0 as *const u32, gl::TRUE);
			gl::DebugMessageInsert(
				gl::DEBUG_SOURCE_APPLICATION,
				gl::DEBUG_TYPE_MARKER,
				1111,
				gl::DEBUG_SEVERITY_NOTIFICATION, -1,
				"Started logging OpenGL messages".as_ptr() as *const i8);
		}

		Context { window_size: (800, 600) }
	}

	pub fn create_texture() -> ! {
		unimplemented!()
	}

	pub fn alloc_buffer_from_data<'a, T>(
		&'a self,
		data: &[T],
		access: BufferAccess,
		binding: BufferBindingHint,
		usage: BufferUsage) -> Buffer<T>
	{
		self.alloc_buffer(data.len(), access, binding, usage, Some(data))
	}

	pub fn alloc_buffer<'a, T>(
		&'a self,
		num_elements: usize,
		access: BufferAccess,
		binding: BufferBindingHint,
		usage: BufferUsage,
		initial_data: Option<&[T]>) -> Buffer<T>
	{
		alloc_buffer(num_elements, access, binding, usage, initial_data)
	}

	pub fn create_frame<'a>(
		&'a self,
		render_target_views: &[RenderTargetView],
		depth_stencil_view: Option<DepthStencilView>) -> Frame
	{
		Frame::new(render_target_views, depth_stencil_view)
	}

	pub fn create_screen_frame<'a>(&'a self, window: &Window) -> Frame
	{
		Frame::default(window.dimensions())
	}

	pub fn event(&self, ev: &Event)
	{
		/*match ev
		{
			&Event::WindowResize(w, h) => self.window_size = (w as i32, h as i32),
			_ => {}
		}*/
	}
}


pub struct Frame
{
	framebuffer: GLuint,
	temporary_buffers: Arena<RawBuffer>,
	viewport: (u32, u32, u32, u32)
}


fn create_framebuffer(color_render_targets: &[RenderTargetView], depth_target: Option<DepthStencilView>) -> GLuint
{
	let mut fbo : GLuint = 0;
	unsafe {
		gl::GenFramebuffers(1, &mut fbo);
		gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

		// bind color attachements
		for (i, rtv) in color_render_targets.iter().enumerate() {
			// TODO support targets other than 2d textures
			// (texture layers, cube map faces, whole cube map, etc.)
			gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as u32, rtv.texture, 0);
		}

		// working with borrows of &Option<&mut T> are a bit awkward
		if let Some(ref depth_target) = depth_target
		{
			gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, depth_target.texture, 0);
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

		gl::DrawBuffers(color_render_targets.len() as GLsizei, draw_buffers[..].as_ptr());
		assert!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE);
	}

	fbo
}

impl Frame
{
	pub fn dimensions(&self) -> (u32, u32)
	{
		(self.viewport.2, self.viewport.3)
	}

	fn new(
		render_target_views: &[RenderTargetView],
		depth_stencil_view: Option<DepthStencilView>) -> Frame
	{
		let fbo = create_framebuffer(render_target_views, depth_stencil_view);

		// TODO check that all dimensions match
		let viewport =
			if !render_target_views.is_empty() {
				render_target_views[0].viewport
			} else {
				if let Some(dsv) = depth_stencil_view {
					dsv.viewport
				} else {
					panic!("Tried to create a frame with no attachements")
				}
			};

		unsafe {
			gl::Viewport(viewport.0 as i32, viewport.1 as i32, viewport.2 as i32, viewport.3 as i32);
		}

		Frame {
			framebuffer: fbo,
			temporary_buffers: Arena::new(),
			viewport: viewport
		}
	}

	fn default(window_size: (u32, u32)) -> Frame
	{
		Frame {
			framebuffer: 0,
			temporary_buffers: Arena::new(),
			viewport: (0, 0, window_size.0, window_size.1)
		}
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
		let buf = alloc_raw_buffer(
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
		let buf = alloc_raw_buffer(
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
		pipeline_state: &PipelineState,
		mesh_part: MeshPart,
		uniform_buffers: &[Binding],
		textures: &[TextureBinding])
	{
		// TODO rebind framebuffer only if necessary
		unsafe {
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
		}
		draw_instanced(vertex_buffer, index_buffer, mesh_part, pipeline_state, uniform_buffers, textures);
	}
}

impl Drop for Frame
{
	fn drop(&mut self)
	{
		unsafe {
			gl::DeleteFramebuffers(1, &mut self.framebuffer);
		}
	}
}
