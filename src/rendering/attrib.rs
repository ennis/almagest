use gl;
use gl::types::*;
use libc::{c_void};
use std::ffi::{CString, CStr};
use std::fmt::{Display, Debug};
use std::string::{ToString};
use nalgebra::*;
use std::mem;
use std::raw;
use rendering::shader::{GLProgram};

#[derive(Copy, Clone, Debug)]
pub enum AttributeType
{
	 // 32x4
    Uint32x4 = 0,
    Sint32x4,
    Float4,
    // 32x3
    Uint32x3,
    Sint32x3,
    Float3,
    // 32x2
    Float2,
    // 16x4
    Uint16x4,
    Sint16x4,
    Unorm16x4,
    Snorm16x4,
    Float16x4,
    // 16x2
    Uint16x2,
    Sint16x2,
    Unorm16x2,
    Snorm16x2,
    Float16x2,
    // 8x4
    Uint8x4,
    Sint8x4,
    Unorm8x4,
    Snorm8x4,
    // 8x3
    Uint8x3,
    Sint8x3,
    Unorm8x3,
    Snorm8x3,
    // 8x2
    Uint8x2,
    Sint8x2,
    Unorm8x2,
    Snorm8x2,
    // 10_10_10_2
    Unorm10x3_1x2,
    Snorm10x3_1x2,
    // Single
    Uint32,
    Sint32,
    Uint16,
    Sint16,
    Unorm16,
    Snorm16,
    //
    Uint8,
    Sint8,
    Unorm8,
    Snorm8,
    Float16,
    Float
}

impl ToString for AttributeType
{
	fn to_string(&self) -> String {
        use std::fmt::Write;
        let mut buf = String::new();
        let _ = buf.write_fmt(format_args!("{:?}", self));
        buf.shrink_to_fit();
        buf
	}
}

impl AttributeType
{
	// TODO: invalid for compressed formats
	pub fn byte_size(self) -> usize
	{
		match self {
			// 32x4
			AttributeType::Uint32x4 => 4 * 4,
			AttributeType::Sint32x4 => 4 * 4,
			AttributeType::Float4 => 4 * 4,
			// 32x3
			AttributeType::Uint32x3 => 4 * 3,
			AttributeType::Sint32x3 => 4 * 3,
			AttributeType::Float3 => 4 * 3,
			// 32x2
			AttributeType::Float2 => 4 * 2,
			// 16x4
			AttributeType::Uint16x4 => 2 * 4,
			AttributeType::Sint16x4 => 2 * 4,
			AttributeType::Unorm16x4 => 2 * 4,
			AttributeType::Snorm16x4 => 2 * 4,
			AttributeType::Float16x4 => 2 * 4,
			// 16x2
			AttributeType::Uint16x2 => 2 * 2,
			AttributeType::Sint16x2 => 2 * 2,
			AttributeType::Unorm16x2 => 2 * 2,
			AttributeType::Snorm16x2 => 2 * 2,
			AttributeType::Float16x2 => 2 * 2,
			// 8x4
			AttributeType::Uint8x4 => 4,
			AttributeType::Sint8x4 => 4,
			AttributeType::Unorm8x4 => 4,
			AttributeType::Snorm8x4 => 4,
			// 8x3
			AttributeType::Uint8x3 => 3,
			AttributeType::Sint8x3 => 3,
			AttributeType::Unorm8x3 => 3,
			AttributeType::Snorm8x3 => 3,
			// 8x2
			AttributeType::Uint8x2 => 2,
			AttributeType::Sint8x2 => 2,
			AttributeType::Unorm8x2 => 2,
			AttributeType::Snorm8x2 => 2,
			// 10_10_10_2
			AttributeType::Unorm10x3_1x2 => 4,
			AttributeType::Snorm10x3_1x2 => 4,
			// Single
			AttributeType::Uint32 => 4,
			AttributeType::Sint32 => 4,
			AttributeType::Uint16 => 2,
			AttributeType::Sint16 => 2,
			AttributeType::Unorm16 => 2,
			AttributeType::Snorm16 => 2,
			//
			AttributeType::Uint8 => 1,
			AttributeType::Sint8 => 1,
			AttributeType::Unorm8 => 1,
			AttributeType::Snorm8 => 1,
			// TODO
			AttributeType::Float16 => 2,
			AttributeType::Float => 4
		}
	}

	// type, num components, normalize
	fn gl_description(self) -> (u32, u8, bool)
	{
		match self {
			// 32x4
			AttributeType::Uint32x4 =>   (gl::UNSIGNED_INT, 4, false),
			AttributeType::Sint32x4 =>   (gl::INT, 4, false),
			AttributeType::Float4 =>     (gl::FLOAT, 4, false),
			// 32x3
			AttributeType::Uint32x3 =>   (gl::UNSIGNED_INT, 3, false),
			AttributeType::Sint32x3 =>   (gl::INT, 3, false),
			AttributeType::Float3 =>     (gl::FLOAT, 3, false),
			// 32x2
			AttributeType::Float2 =>     (gl::FLOAT, 2, false),
			// 16x4
			AttributeType::Uint16x4 =>   (gl::UNSIGNED_SHORT, 4, false),
			AttributeType::Sint16x4 =>   (gl::SHORT, 4, false),
			AttributeType::Unorm16x4 =>  (gl::UNSIGNED_SHORT, 4, true),
			AttributeType::Snorm16x4 =>  (gl::SHORT, 4, true),
			AttributeType::Float16x4 =>  (gl::HALF_FLOAT, 4, false),
			// 16x2
			AttributeType::Uint16x2 =>   (gl::UNSIGNED_SHORT, 2, false),
			AttributeType::Sint16x2 =>   (gl::SHORT, 2, false),
			AttributeType::Unorm16x2 =>  (gl::UNSIGNED_SHORT, 2, true),
			AttributeType::Snorm16x2 =>  (gl::SHORT, 2, true),
			AttributeType::Float16x2 =>  (gl::HALF_FLOAT, 2, false),
			// 8x4
			AttributeType::Uint8x4 =>    (gl::UNSIGNED_BYTE, 4, false),
			AttributeType::Sint8x4 =>    (gl::BYTE, 4, false),
			AttributeType::Unorm8x4 =>   (gl::UNSIGNED_BYTE, 4, true),
			AttributeType::Snorm8x4 =>   (gl::BYTE, 4, true),
			// 8x3
			AttributeType::Uint8x3 =>    (gl::UNSIGNED_BYTE, 3, false),
			AttributeType::Sint8x3 =>    (gl::BYTE, 3, false),
			AttributeType::Unorm8x3 =>   (gl::UNSIGNED_BYTE, 3, true),
			AttributeType::Snorm8x3 =>   (gl::BYTE, 3, true),
			// 8x2
			AttributeType::Uint8x2 =>    (gl::UNSIGNED_BYTE, 2, false),
			AttributeType::Sint8x2 =>    (gl::BYTE, 2, false),
			AttributeType::Unorm8x2 =>   (gl::UNSIGNED_BYTE, 2, true),
			AttributeType::Snorm8x2 =>   (gl::BYTE, 2, true),
			// 10_10_10_2
			AttributeType::Unorm10x3_1x2 => (gl::UNSIGNED_INT_2_10_10_10_REV, 4, true),
			AttributeType::Snorm10x3_1x2 => (gl::INT_2_10_10_10_REV, 4, true),
			// Single
			AttributeType::Uint32 =>     (gl::UNSIGNED_INT, 1, false),
			AttributeType::Sint32 =>     (gl::INT, 1, false),
			AttributeType::Uint16 =>     (gl::UNSIGNED_SHORT, 1, false),
			AttributeType::Sint16 =>     (gl::SHORT, 1, false),
			AttributeType::Unorm16 =>    (gl::UNSIGNED_SHORT, 1, true),
			AttributeType::Snorm16 =>    (gl::SHORT, 1, true),
			//
			AttributeType::Uint8 =>      (gl::UNSIGNED_BYTE, 1, false),
			AttributeType::Sint8 =>      (gl::BYTE, 1, false),
			AttributeType::Unorm8 =>     (gl::UNSIGNED_BYTE, 1, true),
			AttributeType::Snorm8 =>     (gl::BYTE, 1, true),
			AttributeType::Float16 =>    (gl::HALF_FLOAT, 1, false),
			AttributeType::Float =>      (gl::FLOAT, 1, false)
		}
	}
}

pub struct Attribute
{
	pub slot: u32,
	pub ty: AttributeType
}


pub struct InputLayout
{
	pub vao: GLuint,
	pub strides: Vec<i32>
}

impl InputLayout
{
	pub fn new(num_buffers: u32, attribs: &[Attribute]) -> InputLayout
	{
		let mut strides = vec![0i32; num_buffers as usize];
		let nbattr = attribs.len();

		let mut vao : GLuint = 0;

		unsafe {
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
		}

		for i in 0..nbattr as usize
		{
			let a = &attribs[i];
			assert!(a.slot < num_buffers, "Invalid buffer slot specified.");
			let (ty, nb_comp, norm) = a.ty.gl_description();
			unsafe {
				gl::EnableVertexAttribArray(i as u32);
				gl::VertexAttribFormat(i as u32, nb_comp as i32, ty, norm as u8, strides[a.slot as usize] as u32);
				gl::VertexAttribBinding(i as u32, a.slot);
			}
			strides[a.slot as usize] += a.ty.byte_size() as i32;
		}

		for i in 0..(num_buffers as usize)
		{
			assert!(strides[i] != 0, "A buffer slot has no attributes.");
		}

		InputLayout { vao: vao, strides: strides }
	}

	pub fn bind(&self)
	{
		unsafe {
			gl::BindVertexArray(self.vao);
		}
	}
}

impl Drop for InputLayout
{
    fn drop(&mut self)
    {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}
