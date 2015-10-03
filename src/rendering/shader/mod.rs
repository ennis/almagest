mod parser;
mod keywords;
mod gl_program;
mod cache;

use rendering::context::*;
use rendering::sampler::*;
use rendering::attrib::*;
pub use self::keywords::*;
pub use self::gl_program::*;
pub use self::cache::*;
use self::parser::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::path::Path;

#[derive(Copy, Clone)]
pub enum UniformType
{
    Float,
    Float2,
    Float3,
    Float4,
    Mat2,
    Mat3,
    Mat4,
    Mat3x4,
    Mat4x3,
    Int,
    Int2,
    Int3,
    Int4
}

/// Represents a 'uniform' item in a shader
pub struct Uniform
{
    pub name: String,
    pub ty: UniformType
}

impl UniformType
{
    fn from_str(s: &str) -> Option<UniformType>
    {
        match s
        {
            "float" => Some(UniformType::Float),
            "float2" => Some(UniformType::Float2),
            "float3" => Some(UniformType::Float3),
            "float4"=> Some(UniformType::Float4),
            "mat2"=> Some(UniformType::Mat2),
            "mat3"=> Some(UniformType::Mat3),
            "mat4"=> Some(UniformType::Mat4),
            "mat3x4"=> Some(UniformType::Mat3x4),
            "mat4x3"=> Some(UniformType::Mat4x3),
            "int"=> Some(UniformType::Int),
            "int2"=> Some(UniformType::Int2),
            "int3"=> Some(UniformType::Int3),
            "int4"=> Some(UniformType::Int4),
            _ => None
        }
    }
}

/// Represents a 'pass' item in a shader
pub struct Pass
{
    /// Name of the pass
    pub name: String
    // TODO: draw state, defines
}

/// Represents a 'sampler' item in a shader
pub struct Sampler
{
    pub name: String,
    pub desc: Sampler2DDesc
}

/// Compiled pipeline state
/// TODO: move this outside the shader subsystem
pub struct PipelineState
{
    pub config: Keywords,
    pub program: GLProgram,
    pub draw_state: DrawState,
    pub layout: InputLayout
}

struct GLSLInput
{
    slot: u32,
    shader_type: UniformType,
    attrib_type: AttributeType
}

pub struct PipelineStateDesc
{
    // TODO: split draw states
    pub keywords: Keywords,
    pub pass: StdPass,
    pub default_draw_state: DrawState,
    pub sampler_block_base: u32,
    pub uniform_block_base: u32
}

/// Parsed shader
pub struct Shader
{
    /// List of sampler parameters
    pub samplers: Vec<Sampler>,
    /// List of uniform parameters
    pub uniforms: Vec<Uniform>,
    /// List of passes
    pub passes: Vec<Pass>,
    /// GLSL source code of the shader, with #includes replaced
    /// TODO: should be backend-specific
    glsl_source: String,
    /// GLSL version of the shader
    /// Must be reinserted with #version ___
    glsl_version: u32,
    /// Input layout (unpacked shader type, raw type in buffer, buffer slot)
    glsl_input_layout: Vec<GLSLInput>,

    //--------------------------------------
    // This should be somewhere else

    /// Cached result of shader resolution
	forward_pass_unlit_prog: RefCell<Option<Rc<PipelineState>>>,
    /// Cached result of shader resolution
    forward_pass_point_light_prog: RefCell<Option<Rc<PipelineState>>>,
    /// Cached result of shader resolution
    forward_pass_spot_light_prog: RefCell<Option<Rc<PipelineState>>>,
    /// Cached result of shader resolution
    forward_pass_directional_light_prog: RefCell<Option<Rc<PipelineState>>>,
    /// Cached result of shader resolution
    deferred_pass_prog: RefCell<Option<Rc<PipelineState>>>,
    /// Cached result of shader resolution
    shadow_pass_prog: RefCell<Option<Rc<PipelineState>>>,
    /// Cache of all loaded variants of this shader
    cache: RefCell<HashMap<Keywords, Rc<PipelineState>>>
}

impl Shader
{
    pub fn load(source_path: &Path) -> Shader
    {
        trace!("Loading shader {:?}", source_path);
        parse_shader(source_path)
    }

    pub fn make_pipeline_state(&self, desc: &PipelineStateDesc) -> PipelineState
    {
        compile_pipeline_state(self, desc.keywords, desc)
    }
}
