use gl;
use gl::types::*;
use libc::{c_void};
use std::ffi::{CString, CStr};
use std::fmt::{Display, Debug};
use std::string::{ToString};
use nalgebra::*;
use std::mem;
use std::raw;
use context::*;
use shader::{Program};
use buffer::*;
use attrib::*;
use draw::*;


#[derive(Copy, Clone)]
pub struct MeshVertex
{
	pub pos: Vec3<f32>,
	pub norm: Vec3<f32>,
	pub tg: Vec3<f32>
}

impl MeshVertex 
{
	pub fn new(pos: [f32; 3], tex: [f32; 2]) -> MeshVertex
	{
		MeshVertex {
			pos: Vec3::new(pos[0], pos[1], pos[2]),
			norm: Vec3::new(0.0f32, 0.0f32, 0.0f32),
			tg: Vec3::new(0.0f32, 0.0f32, 0.0f32)}
	}
}


pub struct Mesh<'a>
{
	pub vb: Buffer<'a, MeshVertex>,
	pub ib: Option<Buffer<'a, u16>>,
	pub parts: Vec<MeshPart>,
	pub num_vertices: usize,
	pub num_indices: usize
}

impl<'a> Mesh<'a>
{
	pub fn new(
		context: &'a Context,
		primitive_type: PrimitiveType,
		vertices: &[MeshVertex], 
		indices: Option<&[u16]>) -> Mesh<'a>
	{
		let vb = context.alloc_buffer_from_data(
			vertices, 
			BufferAccess::WriteOnly, 
			BufferBindingHint::VertexBuffer, 
			BufferUsage::Static);
		let part = MeshPart{
				primitive_type: primitive_type,
				start_vertex: 0,
				start_index: 0,
				num_vertices: vertices.len() as u32,
				num_indices: if let Some(inner_indices) = indices { inner_indices.len() as u32 } else { 0 }
				};
		if let Some(inner_indices) = indices {
			Mesh { 
				vb: vb, 
				ib: Some(context.alloc_buffer_from_data(
					inner_indices, 
					BufferAccess::WriteOnly, 
					BufferBindingHint::IndexBuffer, 
					BufferUsage::Static)), 
				parts: vec![part],
				num_vertices: part.num_vertices as usize,
				num_indices: part.num_indices as usize
			}
		}
		else {
			Mesh {
				vb: vb,
				ib: None,
				parts: vec![part],
				num_vertices: part.num_vertices as usize,
				num_indices: 0
			}
		}
	}
}

/*
pub fn draw_mesh(mesh: &Mesh, layout: &InputLayout, mesh_part: usize, prog: &Program)
{
	unsafe 
	{
		gl::BindVertexArray(layout.vao);
		gl::UseProgram(prog.obj);
		let vbs = [mesh.vb.raw.obj];
		let offsets = [0];
		let strides : &[i32] = &layout.strides[..];
		gl::BindVertexBuffers(0, 1, vbs.as_ptr(), offsets.as_ptr(), strides.as_ptr());
		
		let part = mesh.parts[mesh_part];

		if let Some(ref ib) = mesh.ib {
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib.raw.obj);
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
}*/