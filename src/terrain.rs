use image::{self, GenericImage, Pixel, GrayImage};
use nalgebra::*;
use scene_data::*;
use rendering::*;
use rendering::shader::*;
use std::path::Path;
use asset_loader::*;
use std::rc::Rc;
use std;

struct TerrainVertex
{
    // position AND texture coordinates
    pos: Vec2<f32>
}

pub struct Terrain<'a>
{
    heightmap_tex: Texture2D,
    heightmap_img: GrayImage,
    vertex_buffer: Buffer<'a, TerrainVertex>,
    scale: f32,
    height_scale: f32
}

pub struct TerrainRenderer
{
    shader: Shader,
    pipeline_state: PipelineState
}

#[derive(Copy, Clone)]
struct TerrainShaderParams
{
    scale: f32,
    height_scale: f32
}

impl<'a> Terrain<'a>
{
    pub fn sample_height(&self, x: f64, y: f64) -> f64
    {
        // TODO sample image
        let (w, h) = self.heightmap_img.dimensions();
        (self.heightmap_img.get_pixel(
            clamp(((x / self.scale as f64) * (w as f64)) as u32, 0, w-1 ),
            clamp(((y / self.scale as f64) * (h as f64)) as u32, 0, h-1 )).channels()[0] as f64) / (std::u8::MAX as f64) * (self.height_scale as f64)
    }

    pub fn new<'b>(context: &'b Context, heightmap: &Path, scale: f32, height_scale: f32) -> Terrain<'b> {
        // create a 2D grid of vertices
        let img = image::open(heightmap).unwrap();
        let (dimx, dimy) = img.dimensions();
        let gray_img = img.as_luma8().expect("Wrong heightmap format");

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

        let heightmap_tex = Texture2D::with_pixels(dimx, dimy, 1, TextureFormat::Unorm8, Some(gray_img));

        Terrain {
            heightmap_tex: heightmap_tex,
            // TODO should not clone here
            heightmap_img: gray_img.clone(),
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
        let pso_desc = PipelineStateDesc {
            keywords: Keywords::empty(),
            pass: StdPass::ForwardBase,
            default_draw_state: DrawState::default(),
            sampler_block_base: 0,
            uniform_block_base: 0
        };
        let shader = Shader::load(Path::new("assets/shaders/terrain.glsl"));
        let pso = shader.make_pipeline_state(&pso_desc);
        TerrainRenderer {
			shader: shader,
            pipeline_state: pso
		}
    }

    pub fn render_terrain(&self, terrain: &Terrain, scene_data: &SceneData, frame: &Frame)
    {
        use num::traits::One;
        terrain.heightmap_tex.bind(0);
		let terrain_params = frame.make_uniform_buffer(&TerrainShaderParams {
            scale: terrain.scale,
            height_scale: terrain.height_scale
        });
		frame.draw(
			terrain.vertex_buffer.raw.as_raw_buf_slice(),
			None,
            &self.shader,
            &self.pipeline_state,
			MeshPart {
                primitive_type: PrimitiveType::Triangle,
                start_vertex: 0,
                start_index: 0,
                num_vertices: 6*terrain.heightmap_tex.dimensions().0*terrain.heightmap_tex.dimensions().1,
                num_indices: 0
            },
			&[
				Binding{slot:0, slice: scene_data.buffer},
                Binding{slot:1, slice: terrain_params.as_raw()}],
			&[]);
    }
}
