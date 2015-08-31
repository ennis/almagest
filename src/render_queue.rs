use gl;
use gl::types::*;
use shader::Program;
use smallvec::SmallVec;
use buffer::{RawBufSlice, Binding};
use draw::MeshPart;
use std::marker::PhantomData;
use std::cell::RefCell;
use frame::Frame;
use attrib::{InputLayout};

/// Per-material states
struct MaterialBlock<'a>
{
	prog: &'a Program,
	uniform_buffers: SmallVec<[Binding<'a>; 8]>
}

#[derive(Copy, Clone, Debug)]
struct MaterialBlockIndex
{
	index: u32
}

// Input mesh block
struct VertexInputBlock<'a>
{
	// TODO replace with mesh?
	vertex_buffers: SmallVec<[Binding<'a>; 8]>,
	index_buffer: Option<RawBufSlice<'a>>,
	input_layout: &'a InputLayout
}

#[derive(Copy, Clone, Debug)]
struct VertexInputBlockIndex
{
	index: u32
}

// 28 bytes
// does not include pass-specific data
struct RenderItem<'a>
{ 
	// 4b
	vertex_input_block_id: u32,
	// 4b
	// RULE: if it's the same material then it should resolve to the same shader 
	// during pass execution
	material_block_id: u32,
	// 4b
	object_block_id: u32,
	// 20b
	// TODO: replace with part-index?
	// with special value for dynamic draw calls
	mesh_part: MeshPart,
	// per-object uniform block
	// 12b
	object_uniforms: Option<Binding<'a>>
}

///
/// 
///
pub struct RenderQueue<'a>
{
	material_blocks: Vec<MaterialBlock<'a>>,
	vertex_input_blocks: Vec<VertexInputBlock<'a>>,
	render_items: Vec<RenderItem<'a>>
}

fn to_small_vec<T: Clone>(elements: &[T]) -> SmallVec<[T; 8]>
{
	let mut v = SmallVec::<[T; 8]>::new();
	v.extend(elements.iter().cloned());
	v
}

impl<'a> RenderQueue<'a> 
{
	pub fn new() -> RenderQueue<'a>
	{
		RenderQueue {
			material_blocks: Vec::new(),
			vertex_input_blocks: Vec::new(),
			render_items: Vec::new()
		}
	}

	// add a material block, return an index
	// TODO not exactly sure about the effect of lifetime annotations here
	pub fn create_material_block(
		&mut self, 
		prog: &'a Program, 
		uniform_buffers: &[Binding<'a>]) -> MaterialBlockIndex
	{
		self.material_blocks.push(
			MaterialBlock {
				prog: prog, 
				uniform_buffers: to_small_vec(uniform_buffers)
			}
		);

		MaterialBlockIndex { 
			index: (self.material_blocks.len()-1) as u32 
		}
	}

	pub fn create_vertex_input_block(
		&mut self, 
		input_layout: &'a InputLayout,
		vertex_buffers: &[Binding<'a>], 
		index_buffer: Option<RawBufSlice<'a>>) -> VertexInputBlockIndex
	{
		self.vertex_input_blocks.push(
			VertexInputBlock {
				vertex_buffers: to_small_vec(vertex_buffers), 
				index_buffer: index_buffer,
				input_layout: input_layout
			}
		);

		VertexInputBlockIndex { 
			index: (self.vertex_input_blocks.len() - 1) as u32 
		}
	}

	// note that this invalidates all previously created material blocks
	pub fn clear(&mut self)
	{
		self.material_blocks.clear();
		self.render_items.clear();
	}

	pub fn add_render_item(
		&mut self, 
		material_block: MaterialBlockIndex,
		vertex_input_block: VertexInputBlockIndex,
		mesh_part: MeshPart,
		object_uniforms: Option<Binding<'a>>)
	{
		self.render_items.push(
			RenderItem {
				vertex_input_block_id: vertex_input_block.index,
				material_block_id: material_block.index,
				object_block_id: 0,
				mesh_part: mesh_part,
				object_uniforms: object_uniforms
			}
		);
	}

	// flush the render queue
	pub fn execute(&mut self, frame: &Frame, pass_uniforms: Option<Binding>)
	{
		for ri in self.render_items.iter()
		{
			let vtx_block = &self.vertex_input_blocks[ri.vertex_input_block_id as usize];
			let mat_block = &self.material_blocks[ri.material_block_id as usize];
			let mut uniform_bindings = SmallVec::<[_; 8]>::new();
			uniform_bindings.extend(mat_block.uniform_buffers.iter().cloned());
			if let Some(obj_uniforms) = ri.object_uniforms {
				uniform_bindings.push(obj_uniforms);
			}
			if let Some(pass_uniforms) = pass_uniforms {
				uniform_bindings.push(pass_uniforms);
			}
			frame.draw(vtx_block.vertex_buffers[0].slice, vtx_block.index_buffer, &vtx_block.input_layout, ri.mesh_part, mat_block.prog, &uniform_bindings[..], &[]);

		}
	}

}