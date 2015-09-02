use gl;
use gl::types::*;
use libc::{c_void};
use std::ffi::{CString, CStr};
use std::fmt::{Display, Debug};
use std::string::{ToString};
use nalgebra::*;
use std::mem;
use std::raw;
use context::{Context};
use std::path::Path;
use std::fs::{File};
use std::io::{BufReader};

#[derive(Copy, Clone)]
pub enum ShaderStage
{
	Vertex,
	Fragment,
	Geometry,
	TessControl,
	TessEval
}

impl ShaderStage
{
	fn to_gl_enum(self) -> GLenum 
	{
		match self
		{
			ShaderStage::Vertex => gl::VERTEX_SHADER,
			ShaderStage::Fragment => gl::FRAGMENT_SHADER,
			ShaderStage::TessEval => gl::TESS_EVALUATION_SHADER,
			ShaderStage::TessControl => gl::TESS_CONTROL_SHADER,
			ShaderStage::Geometry => gl::GEOMETRY_SHADER
		}
	}
}

pub struct Shader {
    obj: GLuint
}

impl Shader
{
	pub fn new(src: &str, stage: ShaderStage) -> Option<Shader>
	{
		unsafe {
			let gl_stage = stage.to_gl_enum();
			let obj = gl::CreateShader(gl_stage);
			let srcs = [src.as_ptr() as *const i8];
			let lens = [src.len() as GLint];
			gl::ShaderSource(obj, 1, &srcs[0] as *const *const i8, &lens[0] as *const GLint);
			gl::CompileShader(obj);
			let mut status: GLint = 0;
			let mut log_size: GLint  = 0;
			gl::GetShaderiv(obj, gl::COMPILE_STATUS, &mut status);
			gl::GetShaderiv(obj, gl::INFO_LOG_LENGTH, &mut log_size);
			trace!("COMPILE_STATUS: log_size: {}, status: {}", log_size, status);
			if status != gl::TRUE as GLint
			{
				error!("Error compiling shader. Compilation log follows.");
				let mut log_buf: Vec<u8> = Vec::with_capacity(log_size as usize);
				gl::GetShaderInfoLog(obj, log_size, &mut log_size, log_buf.as_mut_ptr() as *mut i8);
				log_buf.set_len(log_size as usize);
				error!("{}", String::from_utf8(log_buf).ok().expect("Cannot convert to utf-8"));
				gl::DeleteShader(obj);
				None
			}
			else {
				Some(Shader {obj: obj} )
			}
		}
	}
}

impl Drop for Shader
{
	fn drop(&mut self)
	{
		unsafe {
			gl::DeleteShader(self.obj);
		}
	}
}

fn link_program(obj: GLuint) -> Option<GLuint>
{
	unsafe
	{
		gl::LinkProgram(obj);
		let mut status: GLint = 0;
		let mut log_size: GLint  = 0;
		gl::GetProgramiv(obj, gl::LINK_STATUS, &mut status);
		gl::GetProgramiv(obj, gl::INFO_LOG_LENGTH, &mut log_size);
		trace!("LINK_STATUS: log_size: {}, status: {}", log_size, status);
		if status != gl::TRUE as GLint
		{
			error!("Error linking program. Link log follows.");
			if log_size != 0 {
				let mut log_buf: Vec<u8> = Vec::with_capacity(log_size as usize);
				gl::GetProgramInfoLog(obj, log_size, &mut log_size, log_buf.as_mut_ptr() as *mut i8);
				log_buf.set_len(log_size as usize);
				error!("{}", String::from_utf8(log_buf).ok().expect("Cannot convert to utf-8"));
			} else {
				error!("(No log)");	
			}
			None
		}
		else {
			Some(obj)
		}
	}
}

#[derive(Copy, Clone)]
pub struct ShaderPipelineDesc<'a>
{
	vs: &'a Shader,
	gs: Option<&'a Shader>,
	fs: &'a Shader
}

pub struct Program
{
	pub obj: GLuint
}

impl Program
{
	pub fn from_source(vs_source: &str, ps_source: &str) -> Option<Program>
	{
		let vs = Shader::new(vs_source, ShaderStage::Vertex).unwrap();
		let fs = Shader::new(ps_source, ShaderStage::Fragment).unwrap();
		Program::new(ShaderPipelineDesc { vs: &vs, fs: &fs, gs: None })
	}

	pub fn new(pipeline: ShaderPipelineDesc) -> Option<Program>
	{
		let obj: GLuint;
		unsafe
		{
			obj = gl::CreateProgram();
			gl::AttachShader(obj, pipeline.vs.obj);
			gl::AttachShader(obj, pipeline.fs.obj);
			if let Some(gs) = pipeline.gs {
				gl::AttachShader(obj, gs.obj);
			}
			let result = link_program(obj);
			gl::DetachShader(obj, pipeline.vs.obj);
			gl::DetachShader(obj, pipeline.fs.obj);
			if let Some(gs) = pipeline.gs {
				gl::DetachShader(obj, gs.obj);
			}
			if let Some(_) = result {
				Some(Program {obj: obj})
			}
			else {
				gl::DeleteProgram(obj);
				None
			}
		}
	}
}

pub fn load_shader_source(path: &Path) -> String
{
	use std::io::Read;
	let f = File::open(path).unwrap();
	let mut reader = BufReader::new(&f);
	let mut src = String::new();
	reader.read_to_string(&mut src).unwrap();
	src
}

impl Drop for Program
{
	fn drop(&mut self)
	{
		unsafe {
			gl::DeleteProgram(self.obj);
		}
	}
}
