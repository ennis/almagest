use gl;
use gl::types::*;
use std::mem;
use buffer::{RawBuffer, BufSlice, RawBufSlice, BufferAccess, 
			BufferBindingHint, BufferUsage, 
			BufferAllocator, as_byte_slice, object_as_byte_slice, Binding};
use typed_arena::{Arena};
use std::cell::RefCell;
use attrib::*;
use shader::{Program};
use draw::*;
use texture::Texture2D;


pub struct RenderTarget<'a>
{
	viewport: (i32, i32, i32, i32),
	output: RenderTargetOutput<'a>
}

pub enum RenderTargetOutput<'a>
{
	Screen,
	Texture { color_targets: Vec<&'a mut Texture2D> }
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
	
	pub fn render_to_texture( color_targets: Vec<&'a mut Texture2D> ) -> RenderTarget<'a> 
	{
		// TODO check that all color & depth targets have the same size
		RenderTarget {
			viewport: (0, 0, color_targets[0].width() as i32, color_targets[0].height() as i32),
			output: RenderTargetOutput::Texture { color_targets: color_targets }
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

fn create_framebuffer(color_targets: &[&mut Texture2D]) -> GLuint
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
		
		// no depth texture
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
	pub fn new(buffer_allocator: &'a BufferAllocator, render_target: RenderTarget<'a>) -> Frame<'a>
	{
		// TODO non-random arena size
		let fbo = match render_target.output {
			RenderTargetOutput::Screen => 0,
			RenderTargetOutput::Texture { color_targets: ref color_targets } => create_framebuffer(&color_targets[..])
		};
		Frame {
			buffer_allocator: buffer_allocator,
			temporary_buffers: Arena::with_capacity(300),
			render_target: render_target,
			framebuffer: fbo
		}
	}

	pub fn alloc_temporary_buffer<'b, T>(
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
			if let Some(d) = initial_data { Some(as_byte_slice(d)) } else { None });
			//initial_data.map(|d| as_byte_slice(d)));
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


	pub fn draw(
		&self, 
		vertex_buffer: RawBufSlice,
		index_buffer: Option<RawBufSlice> ,
		layout: &InputLayout, 
		mesh_part: MeshPart,
		prog: &Program,
		uniform_buffers: &[Binding],
		textures: &[&Texture2D])
	{
		// HERE: rebind framebuffer if necessary
		draw_instanced(vertex_buffer, index_buffer, layout, mesh_part, prog, uniform_buffers, textures);
	}
}