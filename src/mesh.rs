use gl;
use gl::types::*;
use libc::{c_void};
use nalgebra::*;
use std::mem;
use std::raw;
use context::*;
use shader::*;
use buffer::*;
use attrib::*;
use draw::*;
use std::path::{Path};
use scene_data::*;
use draw_state::*;
use frame::*;
use tobj;
use material::Material;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MeshVertex
{
	pub pos: Vec3<f32>,
	pub norm: Vec3<f32>,
	pub tg: Vec3<f32>,
	pub tex: Vec2<f32>
}

impl MeshVertex
{
	pub fn new(pos: [f32; 3], tex: [f32; 2]) -> MeshVertex
	{
		MeshVertex {
			pos: Vec3::new(pos[0], pos[1], pos[2]),
			norm: Vec3::new(0.0f32, 0.0f32, 0.0f32),
			tg: Vec3::new(0.0f32, 0.0f32, 0.0f32),
			tex: Vec2::new(tex[0], tex[1])
		}
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

pub struct MeshRenderer
{
	layout: InputLayout,
	prog: Program
}

impl MeshRenderer
{
	pub fn new(context: &Context) -> MeshRenderer
	{
		MeshRenderer {
			layout: InputLayout::new(1, &[
				Attribute{ slot: 0, ty: AttributeType::Float3 },
				Attribute{ slot: 0, ty: AttributeType::Float3 },
				Attribute{ slot: 0, ty: AttributeType::Float3 },
				Attribute{ slot: 0, ty: AttributeType::Float2 }]),
			prog: Program::from_source(
				&load_shader_source(Path::new("assets/shaders/default.vs")),
				&load_shader_source(Path::new("assets/shaders/default.fs"))).expect("Error creating program")
		}
	}

	/// Draw the specified mesh in a scene
	pub fn draw_mesh(&self,
			mesh: &Mesh,
			scene_data: &SceneData,
			material: &Material,
			transform: &Mat4<f32>,
			frame: &Frame)
	{
	 	material.bind();
		let params = frame.make_uniform_buffer(scene_data);
		let model_matrix = frame.make_uniform_buffer(transform);
		frame.draw(
			mesh.vb.raw.as_raw_buf_slice(),
			mesh.ib.as_ref().map(|ib| ib.raw.as_raw_buf_slice()),
			&DrawState::default(),
			&self.layout,
			mesh.parts[0],
			&self.prog,
			&[
				Binding{slot:0, slice: params.as_raw()},
				Binding{slot:1, slice: model_matrix.as_raw()}
			],
			&[]);
	}
}

impl<'a> Mesh<'a>
{
	/// create a mesh from an OBJ file
	pub fn load_from_obj(
		context: &'a Context,
		path: &Path) -> Mesh<'a>
	{
		let mut vertices = Vec::<MeshVertex>::new();
		let mut indices = Vec::<u16>::new();
		let (models, materials) = tobj::load_obj(path).unwrap();

		let ref m = models[0].mesh;

		for i in 0..m.indices.len() {
			indices.push(m.indices[i] as u16);
		}


		// mesh has texture coordinates
		if m.texcoords.len() > 0 {
			//println!("texcoords {} positions {}", m.texcoords.len(), m.positions.len());
			//trace!("Mesh has texcoords!");
			for i in 0..m.positions.len() / 3 {
				vertices.push(MeshVertex {
					pos: Vec3::new(m.positions[3*i],
									m.positions[3*i+1],
									m.positions[3*i+2]),
					norm: Vec3::new(m.normals[3*i],
									m.normals[3*i+1],
									m.normals[3*i+2]),
					tg: Vec3::new(0.0, 0.0, 0.0),
					tex: Vec2::new(m.texcoords[2*i], m.texcoords[2*i+1])
				});
				//trace!("{},{}", m.texcoords[2*i], m.texcoords[2*i+1]);
			}
		} else {
			// mesh doesn't have texture coordinates
			for i in 0..m.positions.len() / 3 {
				vertices.push(MeshVertex {
					pos: Vec3::new(m.positions[3*i],
									m.positions[3*i+1],
									m.positions[3*i+2]),
					norm: Vec3::new(m.normals[3*i],
									m.normals[3*i+1],
									m.normals[3*i+2]),
					tg: Vec3::new(0.0, 0.0, 0.0),
					tex: Vec2::new(m.positions[3*i], m.positions[3*i+1])
				});
			}
		}

		Mesh::new(context, PrimitiveType::Triangle,
			&vertices[..],
			Some(&indices[..]))
	}

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
