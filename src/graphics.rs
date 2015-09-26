use rendering::*;
use libc::{c_void};
use nalgebra::*;
use std::mem;
use std::raw;
use std::path::{Path};
use scene_data::*;
use tobj;
use material::Material;
use shadow_pass::*;
use image::{self, GenericImage};
use asset_loader::*;
use rendering::shader::*;
use std::rc::Rc;

pub struct Rect
{
	pub top: f32,
	pub bottom: f32,
	pub left: f32,
	pub right: f32
}

impl Rect
{
    pub fn from_dimensions(x: f32, y: f32, width: f32, height: f32) -> Rect
    {
        Rect { top: y, left: x, right: x + width, bottom: y + height }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MeshVertex
{
	pub pos: [f32; 3],
	pub norm: [f32; 3],
	pub tg: [f32; 3],
	pub tex: [f32; 2]
}

impl MeshVertex
{
	pub fn new(pos: [f32; 3], tex: [f32; 2]) -> MeshVertex
	{
		MeshVertex {
			pos: pos,
			norm: [0.0; 3],
			tg: [0.0; 3],
			tex: tex
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
					pos: [m.positions[3*i], m.positions[3*i+1], m.positions[3*i+2]],
					norm: [m.normals[3*i], m.normals[3*i+1], m.normals[3*i+2]],
					tg: [0.0; 3],
					tex: [m.texcoords[2*i], m.texcoords[2*i+1]]
				});
				//trace!("{},{}", m.texcoords[2*i], m.texcoords[2*i+1]);
			}
		} else {
			// mesh doesn't have texture coordinates
			for i in 0..m.positions.len() / 3 {
				vertices.push(MeshVertex {
					pos: [m.positions[3*i], m.positions[3*i+1], m.positions[3*i+2]],
					norm: [m.normals[3*i], m.normals[3*i+1], m.normals[3*i+2]],
					tg: [0.0; 3],
					tex: [m.positions[3*i], m.positions[3*i+1]]    // dummy texture coordinates
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

// shared rendering resources
pub struct Graphics<'a>
{
    context: &'a Context,
	// Default sampler
	default_sampler: Sampler2D,

    // Shared programs
    blit_shader: Shader,
    default_shader: Shader,
	blit_pso: Rc<PipelineState>,
	default_pso: Rc<PipelineState>,
    // default (missing) texture (material)
    missing_tex: Texture2D
}

fn load_texture2d_from_file(path: &Path) -> Texture2D
{
    let img = image::open(path).unwrap();
    let (w, h) = img.dimensions();
    let img2 = img.as_rgb8().unwrap();
    Texture2D::with_pixels(w, h, 1, TextureFormat::Unorm8x3, Some(img2))
}

impl<'a> Graphics<'a>
{
    pub fn new(context: &'a Context) -> Graphics<'a>
    {
        //...
		let mut shader_cache = ShaderCache::new();

		let pso_desc = PipelineStateDesc {
			keywords: Keywords::empty(),
			pass: StdPass::ForwardBase,
			default_draw_state: DrawState::default(),
			sampler_block_base: 0,
			uniform_block_base: 0
		};

		let wire_pso_desc = PipelineStateDesc {
			keywords: Keywords::empty(),
			pass: StdPass::ForwardBase,
			default_draw_state: DrawState { polygon_fill_mode: PolygonFillMode::Wireframe, .. DrawState::default() },
			sampler_block_base: 0,
			uniform_block_base: 0
		};

		let blit_shader = Shader::load(Path::new("assets/shaders/blit.glsl"));
		let mesh_shader = Shader::load(Path::new("assets/shaders/debug.glsl"));
		let blit_pso = shader_cache.get(&blit_shader, &pso_desc);
		let mesh_pso = shader_cache.get(&mesh_shader, &wire_pso_desc);

        Graphics {
            context: context,
            // blitter
			blit_shader: blit_shader,
			default_shader: mesh_shader,
			blit_pso: blit_pso,
			default_pso: mesh_pso,
            missing_tex: load_texture2d_from_file(Path::new("assets/img/missing_512.png")),
			default_sampler: Sampler2DDesc::default().build()
        }
    }

    /// Get the default texture
    pub fn default_texture(&self) -> &Texture2D
    {
        &self.missing_tex
    }

    /// returns the underlying context
    /// TODO: mutable reference? borrow_context?
    pub fn context(&self) -> &Context
    {
        self.context
    }

	/// Draw a mesh with the specified shader and parameters
	pub fn draw_mesh_with_shader(&self, mesh: &Mesh, shader: &Shader, pipeline_state: &PipelineState, bindings: &[Binding], frame: &Frame)
	{
		frame.draw(
			mesh.vb.raw.as_raw_buf_slice(),
			mesh.ib.as_ref().map(|ib| ib.raw.as_raw_buf_slice()),
			&shader,
			&pipeline_state,
			mesh.parts[0],
			bindings,
			&[]);
	}

    /// Draw a mesh in wireframe
    pub fn draw_wire_mesh(&self, mesh: &Mesh, bindings: &[Binding], frame: &Frame)
    {
		frame.draw(
			mesh.vb.raw.as_raw_buf_slice(),
			mesh.ib.as_ref().map(|ib| ib.raw.as_raw_buf_slice()),
			&self.default_shader,
			&self.default_pso,
			mesh.parts[0],
			bindings,
			&[]);
    }

    /// Blit a texture in the frame
    pub fn blit(&self, texture: &Texture2D, rect: &Rect, frame: &Frame)
    {
        #[derive(Copy, Clone)]
        #[repr(C)]
        struct Vertex2D {
            pos: [f32; 2],
            tex: [f32; 2]
        }

        #[derive(Copy, Clone)]
        #[repr(C)]
        struct BlitData {
            viewport_size: [f32; 2]
        };

        let (width, height) = frame.dimensions();

        let buf = frame.alloc_temporary_buffer(6, BufferBindingHint::VertexBuffer, Some(&[
            Vertex2D { pos : [rect.left, rect.top], tex : [0.0, 0.0] },
            Vertex2D { pos : [rect.right, rect.top],  tex: [1.0, 0.0] },
            Vertex2D { pos : [rect.left, rect.bottom], tex: [0.0, 1.0] },
            Vertex2D { pos : [rect.left, rect.bottom], tex: [0.0, 1.0] },
            Vertex2D { pos : [rect.right, rect.top], tex: [1.0, 0.0] },
            Vertex2D { pos : [rect.right, rect.bottom], tex: [1.0, 1.0] }
            ]));
        let buf_2 = frame.make_uniform_buffer(&BlitData {
                viewport_size: [width as f32, height as f32]
            });

        frame.draw(
            buf.as_raw(),
            None,
            &self.blit_shader,
			&self.blit_pso,
            MeshPart {
                primitive_type: PrimitiveType::Triangle,
                start_vertex: 0,
                start_index: 0,
                num_vertices: 6,
                num_indices: 0
                },
            &[Binding {slot:0, slice: buf_2.as_raw() }],
            &[TextureBinding {slot: 0, sampler: &self.default_sampler, texture: &texture}]
            );
    }

    /// Blit a part of a texture in the frame
    pub fn blit_part(&self, frame: &Frame)
    {
        // TODO
    }
}
