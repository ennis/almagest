use super::parser::*;
use super::keywords::*;
use super::PipelineState;
use rendering::frame::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;
use super::{Shader, UniformType};
use super::gl_program::*;

pub struct ShaderCache;



pub struct ShaderCacheQuery
{
    pub keywords: Keywords,
    pub pass: StdPass,
    // NOTE: this does not participate in shader resolution
    pub default_draw_state: DrawState,
    // NOTE: this does not participate in shader resolution
    pub sampler_block_base: u32,
    // NOTE: this does not participate in shader resolution
    pub uniform_block_base: u32
}

macro_rules! keyword_impl { ($y:expr, $kw:expr, $x:ident) => { if $y.contains($x) { $kw.push( stringify!($x) ) }; } }

fn variant_bits_to_keywords(config: Keywords) -> Vec<&'static str>
{
	let mut kw = Vec::new();
	keyword_impl!(config, kw, FORWARD_BASE);
	keyword_impl!(config, kw, FORWARD_ADD);
	keyword_impl!(config, kw, POINT_LIGHT);
	keyword_impl!(config, kw, DIRECTIONAL_LIGHT);
	keyword_impl!(config, kw, SPOT_LIGHT);
	keyword_impl!(config, kw, SHADOWS_SIMPLE);
	keyword_impl!(config, kw, SHADOW);
	keyword_impl!(config, kw, DEFERRED);
	kw
}

fn shader_type_to_glsl(ty: UniformType) -> &'static str
{
    match ty
    {
        UniformType::Float => "float",
        UniformType::Float2 => "vec2",
        UniformType::Float3 => "vec3",
        UniformType::Float4 => "vec4",
        UniformType::Mat2 => "mat2",
        UniformType::Mat3 => "mat3",
        UniformType::Mat4 => "mat4",
        UniformType::Mat3x4 => "mat3x4",
        UniformType::Mat4x3 => "mat4x3",
        UniformType::Int => "int",
        UniformType::Int2 => "ivec2",
        UniformType::Int3 => "ivec3",
        UniformType::Int4 => "ivec4"
    }
}


fn compile_program(shader: &Shader, config: Keywords, query: &ShaderCacheQuery) -> GLProgram
{
    let keywords = variant_bits_to_keywords(config);
    let mut out = Vec::<u8>::new();
	for kw in keywords.iter() {
		writeln!(out, "#define {}", kw).unwrap();
	}
    // make the material block
    if !shader.uniforms.is_empty() {
        writeln!(out, r"layout(std140, binding = {}) uniform MaterialBlock {{", query.uniform_block_base);
        for u in shader.uniforms.iter() {
            writeln!(out, "{} {};", shader_type_to_glsl(u.ty), u.name);
        }
        writeln!(out, "}};");
    }
    writeln!(out, "{}", &shader.glsl_source[..]);

	let mut out_vs = Vec::<u8>::new();
	writeln!(out_vs, "#version {}", shader.glsl_version).unwrap();
	writeln!(out_vs, "#define _VERTEX_").unwrap();
	out_vs.push_all(&out[..]);
	let mut out_fs = Vec::<u8>::new();
	writeln!(out_fs, "#version {}", shader.glsl_version).unwrap();
	writeln!(out_fs, "#define _FRAGMENT_").unwrap();
	out_fs.push_all(&out[..]);

	let (vs, fs) = (String::from_utf8(out_vs).unwrap(), String::from_utf8(out_fs).unwrap());

    trace!("{}", &vs[..]);

    GLProgram::from_source(&vs[..], &fs[..]).unwrap()
}

impl ShaderCache
{
    pub fn new() -> ShaderCache
    {
        ShaderCache
    }

    fn load_variant(&mut self, shader: &Shader, config: Keywords, query: &ShaderCacheQuery) -> Rc<PipelineState>
	{
		shader.cache.borrow_mut().entry(config)
				.or_insert_with(|| Rc::new(
					PipelineState {
                        draw_state: query.default_draw_state,
						config: config,
						program: compile_program(shader, config, query)})).clone()
	}

    // helper method
    fn get_and_cache_variant(
        &mut self,
        shader: &Shader,
        variant: &RefCell<Option<Rc<PipelineState>>>,
        config: Keywords,
        query: &ShaderCacheQuery) -> Rc<PipelineState>
    {
        if let Some(ref variant) = *(variant.borrow())
        {
            if variant.config == config {
                return variant.clone();
            }
        }

        // wrong config or config not loaded yet, reload and cache
        let result = self.load_variant(shader, config, query);
        *(variant.borrow_mut()) = Some(result.clone());
        result
    }

    pub fn get(&mut self, shader: &Shader, query: &ShaderCacheQuery) -> Rc<PipelineState>
    {
        // Add pass specific keyword
        let config = query.keywords | match query.pass {
            StdPass::ForwardBase => FORWARD_BASE,
            StdPass::ForwardAdd => FORWARD_ADD,
            StdPass::Deferred => DEFERRED,
            StdPass::Shadow => SHADOW
        };

        if config.contains(FORWARD_BASE | POINT_LIGHT) {
            self.get_and_cache_variant(shader, &shader.forward_pass_point_light_prog, config, query)
        } else if config.contains(FORWARD_BASE | DIRECTIONAL_LIGHT) {
            self.get_and_cache_variant(shader, &shader.forward_pass_directional_light_prog, config, query)
        } else if config.contains(FORWARD_BASE | SPOT_LIGHT) {
            self.get_and_cache_variant(shader, &shader.forward_pass_spot_light_prog, config, query)
        } else if config.contains(SHADOW) {
            self.get_and_cache_variant(shader, &shader.shadow_pass_prog, config, query)
        } else {
            self.get_and_cache_variant(shader, &shader.forward_pass_unlit_prog, config, query)
        }
    }

}
