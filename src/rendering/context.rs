use rendering::buffer::{Buffer, BufferAccess, BufferBindingHint, BufferUsage, BufferAllocator};
use rendering::shader::{GLShader, GLProgram, ShaderStage, ShaderPipelineDesc};
use rendering::frame::{Frame, RenderTarget};
use gl::types::*;
use gl;
use libc::c_void;
use std::ffi::CStr;
use event::*;

// TODO the context should know the window dimensions
// (and be notified when the window size changes)
pub struct Context
{
	buffer_allocator: BufferAllocator,
	// three previous frames
	//last_frames: [Option<Frame<'a>>; 3]
	// TODO ref to window
}

// Views: persistent or scoped?
// Persistent: like D3D, create the view object only once (borrowing the texture, but not mutably)
// *** Scoped: the view object is created only when it is needed, and dropped when it is not needed anymore.
//     for render targets, this allows the object to 'lock' the underlying texture for writing, preventing
//     it to be bound as a texture at the same time (undefined behavior in OpenGL)

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
		trace!("{}", msg_str.to_str().unwrap());
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

		Context { buffer_allocator: BufferAllocator }
	}

	pub fn create_texture() -> ! {
		unimplemented!()
	}

	pub fn alloc_buffer_from_data<'a, T>(
		&'a self,
		data: &[T],
		access: BufferAccess,
		binding: BufferBindingHint,
		usage: BufferUsage) -> Buffer<'a, T>
	{
		self.alloc_buffer(data.len(), access, binding, usage, Some(data))
	}

	pub fn alloc_buffer<'a, T>(
		&'a self,
		num_elements: usize,
		access: BufferAccess,
		binding: BufferBindingHint,
		usage: BufferUsage,
		initial_data: Option<&[T]>) -> Buffer<'a, T>
	{
		self.buffer_allocator.alloc_buffer(num_elements, access, binding, usage, initial_data)
	}

	pub fn create_frame<'a>(&'a self, render_target: RenderTarget<'a>) -> Frame<'a>
	{
		Frame::new(&self.buffer_allocator, render_target)
	}

	pub fn event(&self, ev: &Event)
	{
		// Nothing to do (yet)
	}
}
