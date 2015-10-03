use std::fs::{File};
use std::path::{PathBuf, Path};
use std::io::{BufReader, BufWriter, BufRead, Read, Write};
use rendering::sampler::Sampler2DDesc;
use std::rc::{Rc};
use std::cell::{RefCell};
use std::str;
use rendering::context::DrawState;
use std::collections::HashMap;
use super::{Uniform, Pass, Sampler, UniformType, GLSLInput};
use rendering::attrib::*;
use super::Shader;

//==========================================================
// Shader syntax

#[derive(Debug, PartialEq, Eq)]
pub enum ShConfigKw<'a> {
    With(&'a str),
    Without(&'a str)
}

#[derive(Debug, PartialEq, Eq)]
pub struct ShUniform<'a> {
    name: &'a str,
    ty: &'a str
}

#[derive(Debug, PartialEq, Eq)]
pub struct ShPass<'a>
{
    name: &'a str
}

pub enum ShShaderItem<'a>
{
    Sampler(Box<ShSampler<'a>>),
    Uniform(Box<ShUniform<'a>>),
    Pass(Box<ShPass<'a>>),
    Layout(Box<Vec<(&'a str, u32)>>)
}

pub struct ShSampler<'a>
{
    name: &'a str,
    desc: Sampler2DDesc
}

pub struct ShShaderConfig<'a>
{
    items: Vec<ShShaderItem<'a>>
}

pub enum ShTextureAddressMode
{
    Clamp,
    Repeat,
    Mirror
}

pub enum ShTextureFilter
{
    Nearest,
    Linear
}

peg_file! sh_grammar("grammar.rustpeg");


#[test]
fn test_sh_grammar_file()
{
    Shader::load(Path::new("assets/shaders/example.glsl"));
}


pub fn parse_shader(source_path: &Path) -> Shader
{
    use std::io::{stderr, stdout};
    use combine::*;
    let f = File::open(&source_path).unwrap();
    let mut reader = BufReader::new(&f);
    let mut source_str = String::new();
    reader.read_to_string(&mut source_str).unwrap();

    let (items, glsl) = sh_grammar::shader_source(&source_str[..]).expect("Shader parse error");
    let mut glsl_pp = String::new();
    let include_paths = [];
    let mut glsl_version = None;
    // preprocess the rest of the file
    process_includes(glsl, &mut glsl_pp, source_path, 0, &include_paths[..], &mut stderr(), &mut glsl_version, None);

    let mut samplers = Vec::new();
    let mut uniforms = Vec::new();
    let mut passes = Vec::new();
    let mut inputs = Vec::new();

    // process configs
    for item in items
    {
        match item
        {
            ShShaderItem::Sampler(s) => samplers.push(Sampler { name: s.name.to_string(), desc: s.desc }),
            ShShaderItem::Uniform(u) => uniforms.push(Uniform {
                name: u.name.to_string(),
                ty: UniformType::from_str(u.ty).expect("Unrecognized uniform type")}),
            ShShaderItem::Pass(p) => passes.push( Pass {
                name: p.name.to_string()
            }),
            ShShaderItem::Layout(glsl_inputs) => {
                if !inputs.is_empty() {
                    panic!("Duplicate glsl_input directive");
                }
                for &(tyname, slot) in glsl_inputs.iter()
                {
                    let (shader_ty, attr_ty) = parse_input_type(tyname);
                    inputs.push( GLSLInput {
                        slot: slot,
                        shader_type: shader_ty,
                        attrib_type: attr_ty
                    });
                }
            }
        }
    }

    Shader {
        samplers: samplers,
        uniforms: uniforms,
        passes: passes,
        glsl_source: glsl_pp,
        glsl_version: glsl_version.unwrap_or(110),
        glsl_input_layout: inputs,
        forward_pass_unlit_prog: RefCell::new(None),
        forward_pass_point_light_prog: RefCell::new(None),
        forward_pass_spot_light_prog: RefCell::new(None),
        forward_pass_directional_light_prog: RefCell::new(None),
        deferred_pass_prog: RefCell::new(None),
        shadow_pass_prog: RefCell::new(None),
        cache: RefCell::new(HashMap::new())
    }
}

//==========================================================
// Include preprocessor
struct ShIncludeFile<'a>
{
	line_number: u32,
	include_depth: u32,
	source_path: PathBuf,
	parent_file: Option<&'a ShIncludeFile<'a>>
}

fn parse_input_type(ty: &str) -> (UniformType, AttributeType)
{
    match ty
    {
        "float" => (UniformType::Float, AttributeType::Float),
        "float2" => (UniformType::Float2, AttributeType::Float2),
        "float3" => (UniformType::Float3, AttributeType::Float3),
        "float4" => (UniformType::Float4, AttributeType::Float4),
        _ => panic!("Invalid input type {}", ty)
    }
}

fn process_includes<W: Write>(
    input: &str,
    output: &mut String,
    source_path: &Path,
    include_depth: u32,
    include_paths: &[&Path],
    out_info_log: &mut W,
    glsl_version: &mut Option<u32>,
    parent_file: Option<&ShIncludeFile>)
{
    use combine::*;
    use std::fmt::Write;

    for (line_number, l) in input.lines().enumerate() {
        if l.len() > 0 {
            // try to parse a pragma preprocessor line
            let ppline = sh_grammar::pragma_include(&l[..]);

            if let Ok(path) = ppline {
                // we found a line with an include directive
                // find the file
                // first, look in the same directory

                let mut resolved = None;
                let parent_dir = source_path.parent().unwrap();
                let local = parent_dir.join(&path);
                if let Ok(f) = File::open(&local)
                {
                    resolved = Some((local, f));
                }
                else
                {
                    // not found, look in system include paths
                    for sys_inc_path in include_paths.iter()
                    {
                        let p = sys_inc_path.join(&path);
                        if let Ok(f) = File::open(&p) {
                            resolved = Some((p, f));
                        }
                    }
                }

                if let Some((p, f)) = resolved {
                    writeln!(output, "//====== INCLUDE FILE {} FROM {}", p.to_str().unwrap(), source_path.to_str().unwrap()).unwrap();
                    writeln!(output, "#line 1 {}", include_depth).unwrap();
                    let mut reader = BufReader::new(&f);
                    let mut inc_source = String::new();
                    reader.read_to_string(&mut inc_source).unwrap();
                    let this_file = ShIncludeFile {
                        line_number: line_number as u32,
                    	include_depth: include_depth,
                    	source_path: source_path.to_path_buf(),
                    	parent_file: parent_file
                    };
                    process_includes(&inc_source[..], output, &p, include_depth+1, include_paths, out_info_log, glsl_version, Some(&this_file));
                }
                else {
                    panic!("Include file not found.")
                }
            }
            else if let Ok((version, _)) = {
                    // TODO: not very readable
                    // TODO: correct parser (fail when extraneous chars are present after version number)
                    let p_version_num = many1(digit()).map(|string: String| string.parse::<u32>().unwrap());
                    char('#').with(spaces())
                             .with(string("version"))
                             .with(skip_many1(space()))
                             .with(p_version_num)
                             .parse(&l[..])
                }
            {
                // we found a #version directive
                if let &mut Some(version) = glsl_version {
                    warn!("Duplicate #version directive, line {}; ignoring.", line_number);
                }
                *glsl_version = Some(version);
            }
            else {
                // XXX fix line endings when not on windows?
                writeln!(output, "{}", l).unwrap();
            }
        }
    }
}
