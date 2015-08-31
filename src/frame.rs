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
use render_target::*;
use texture::Texture2D;

pub struct Frame<'a> 
{
	buffer_allocator: &'a BufferAllocator,
	temporary_buffers: Arena<RawBuffer<'a>>,
	render_target: RenderTarget<'a>
}

impl<'a> Frame<'a>
{
	pub fn new(buffer_allocator: &'a BufferAllocator, render_target: RenderTarget<'a>) -> Frame<'a>
	{
		// TODO non-random arena size
		Frame {
			buffer_allocator: buffer_allocator,
			temporary_buffers: Arena::with_capacity(300),
			render_target: render_target
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
		trace!("sizeof<t>: {}", mem::size_of::<T>());	
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