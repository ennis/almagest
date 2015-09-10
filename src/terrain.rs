use image::{self, GenericImage, Pixel};
use nalgebra::*;
use scene_data::*;
use rendering::*;
use std::path::Path;
use asset_loader::*;
use std::rc::Rc;

struct TerrainVertex
{
    // position AND texture coordinates
    pos: Vec2<f32>
}

pub struct Terrain<'a>
{
    heightmap: Texture2D,
    vertex_buffer: Buffer<'a, TerrainVertex>,
    scale: f32,
    height_scale: f32
}

pub struct TerrainRenderer
{
    prog: Program,
    layout: InputLayout
}

#[derive(Copy, Clone)]
struct TerrainShaderParams
{
    scale: f32,
    height_scale: f32
}

impl<'a> Terrain<'a>
{
    pub fn new<'b>(context: &'b Context, heightmap: &Path, scale: f32, height_scale: f32) -> Terrain<'b> {
        // create a 2D grid of vertices
        let img = image::open(heightmap).unwrap();
        let (dimx, dimy) = img.dimensions();

        let mut vertices = Vec::<TerrainVertex>::with_capacity((6*(dimx-1)*(dimy-1)) as usize);

        for i in 0..(dimy-1) {
            for j in 0..(dimx-1) {
                vertices.push(TerrainVertex {
                    pos: Vec2::new(j as f32 / (dimx-1) as f32, i as f32 / (dimy-1) as f32)
                });
                vertices.push(TerrainVertex {
                    pos: Vec2::new((j+1) as f32 / (dimx-1) as f32, i as f32 / (dimy-1) as f32)
                });
                vertices.push(TerrainVertex {
                    pos: Vec2::new(j as f32 / (dimx-1) as f32, (i+1) as f32 / (dimy-1) as f32)
                });
                vertices.push(TerrainVertex {
                    pos: Vec2::new(j as f32 / (dimx-1) as f32, (i+1) as f32 / (dimy-1) as f32)
                });
                vertices.push(TerrainVertex {
                    pos: Vec2::new((j+1) as f32 / (dimx-1) as f32, i as f32 / (dimy-1) as f32)
                });
                vertices.push(TerrainVertex {
                    pos: Vec2::new((j+1) as f32 / (dimx-1) as f32, (i+1) as f32 / (dimy-1) as f32)
                });
            }
        }

        let buf = context.alloc_buffer_from_data(
            &vertices[..],
            BufferAccess::WriteOnly,
            BufferBindingHint::VertexBuffer,
            BufferUsage::Static);



		let img2 = img.as_rgb8().unwrap();
        let heightmap_tex = Texture2D::with_pixels(dimx, dimy, 1, TextureFormat::Unorm8x3, Some(img2));

        Terrain {
            heightmap: heightmap_tex,
            vertex_buffer: buf,
            height_scale: height_scale,
            scale: scale
        }
    }
}

impl TerrainRenderer
{
    pub fn new() -> TerrainRenderer
    {
        TerrainRenderer {
			layout: InputLayout::new(1, &[Attribute{ slot: 0, ty: AttributeType::Float2 }]),
			prog: Program::from_source(
				&load_shader_source(Path::new("assets/shaders/terrain.vs")),
				&load_shader_source(Path::new("assets/shaders/terrain.fs"))).expect("Error creating program")
		}
    }

    pub fn render_terrain(&self, terrain: &Terrain, scene_data: &SceneData, frame: &Frame)
    {
        use num::traits::One;
        terrain.heightmap.bind(0);
		let terrain_params = frame.make_uniform_buffer(&TerrainShaderParams {
            scale: terrain.scale,
            height_scale: terrain.height_scale
        });
		frame.draw(
			terrain.vertex_buffer.raw.as_raw_buf_slice(),
			None,
			&DrawState::default(),
			&self.layout,
			MeshPart {
                primitive_type: PrimitiveType::Triangle,
                start_vertex: 0,
                start_index: 0,
                num_vertices: 6*terrain.heightmap.dimensions().0*terrain.heightmap.dimensions().1,
                num_indices: 0
            },
			&self.prog,
			&[
				Binding{slot:0, slice: scene_data.buffer},
                Binding{slot:1, slice: terrain_params.as_raw()}],
			&[]);
    }
}
