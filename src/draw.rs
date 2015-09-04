use gl;
use gl::types::*;
use std::mem;
use buffer::{RawBuffer, BufSlice, RawBufSlice, BufferAccess, Binding,
			BufferBindingHint, BufferUsage,
			BufferAllocator, as_byte_slice,
			bind_vertex_buffers,
			bind_index_buffer,
			bind_uniform_buffers};
use typed_arena::{Arena};
use std::cell::RefCell;
use attrib::*;
use shader::{Program};
use texture::Texture2D;

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
		layout: &InputLayout,
		part: MeshPart,
		prog: &Program,
		uniform_buffers: &[Binding],
		textures: &[&Texture2D])
{
	unsafe
	{
		gl::UseProgram(prog.obj);
		bind_uniform_buffers(uniform_buffers);
		bind_vertex_buffers(layout, &[vertex_buffer]);

		for (i,t) in textures.iter().enumerate() {
			t.bind(i as u32);
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
