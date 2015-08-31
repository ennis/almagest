use gl;
use gl::types::*;
use std::mem;

#[derive(Copy, Clone, Debug)]
// Note: same format as vertex attributes, for now
// TODO: add compressed texture formats
// TODO: add depth formats
pub enum TextureFormat
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

impl TextureFormat
{
	// TODO: invalid for compressed formats
	pub fn byte_size(self) -> usize
	{
		match self {
			// 32x4
			TextureFormat::Uint32x4 => 4 * 4,
			TextureFormat::Sint32x4 => 4 * 4,
			TextureFormat::Float4 => 4 * 4,
			// 32x3
			TextureFormat::Uint32x3 => 4 * 3,
			TextureFormat::Sint32x3 => 4 * 3,
			TextureFormat::Float3 => 4 * 3,
			// 32x2
			TextureFormat::Float2 => 4 * 2,
			// 16x4
			TextureFormat::Uint16x4 => 2 * 4,
			TextureFormat::Sint16x4 => 2 * 4,
			TextureFormat::Unorm16x4 => 2 * 4,
			TextureFormat::Snorm16x4 => 2 * 4,
			TextureFormat::Float16x4 => 2 * 4,
			// 16x2
			TextureFormat::Uint16x2 => 2 * 2,
			TextureFormat::Sint16x2 => 2 * 2,
			TextureFormat::Unorm16x2 => 2 * 2,
			TextureFormat::Snorm16x2 => 2 * 2,
			TextureFormat::Float16x2 => 2 * 2,
			// 8x4
			TextureFormat::Uint8x4 => 4,
			TextureFormat::Sint8x4 => 4,
			TextureFormat::Unorm8x4 => 4,
			TextureFormat::Snorm8x4 => 4,
			// 8x3
			TextureFormat::Uint8x3 => 3,
			TextureFormat::Sint8x3 => 3,
			TextureFormat::Unorm8x3 => 3,
			TextureFormat::Snorm8x3 => 3,
			// 8x2
			TextureFormat::Uint8x2 => 2,
			TextureFormat::Sint8x2 => 2,
			TextureFormat::Unorm8x2 => 2,
			TextureFormat::Snorm8x2 => 2,
			// 10_10_10_2
			TextureFormat::Unorm10x3_1x2 => 4,
			TextureFormat::Snorm10x3_1x2 => 4,
			// Single
			TextureFormat::Uint32 => 4,
			TextureFormat::Sint32 => 4,
			TextureFormat::Uint16 => 2,
			TextureFormat::Sint16 => 2,
			TextureFormat::Unorm16 => 2,
			TextureFormat::Snorm16 => 2,
			//
			TextureFormat::Uint8 => 1,
			TextureFormat::Sint8 => 1,
			TextureFormat::Unorm8 => 1,
			TextureFormat::Snorm8 => 1,
			// TODO
			TextureFormat::Float16 => 2,
			TextureFormat::Float => 4
		}
	}

	// num components, internalFormat, externalFormat, externalType
	fn gl_description(self) -> (u8, GLenum, GLenum, GLenum)
	{
		match self {
			// 32x4
			TextureFormat::Uint32x4 =>   (4, gl::RGBA32UI, gl::RGBA_INTEGER, gl::UNSIGNED_INT),
			TextureFormat::Sint32x4 =>   (4, gl::RGBA32I, gl::RGBA_INTEGER, gl::INT),
			TextureFormat::Float4 =>     (4, gl::RGBA32F, gl::RGBA, gl::FLOAT),
			// 32x3
			TextureFormat::Uint32x3 =>   (3, gl::RGB32UI, gl::RGB_INTEGER, gl::UNSIGNED_INT),
			TextureFormat::Sint32x3 =>   (3, gl::RGB32I, gl::RGB_INTEGER, gl::INT),
			TextureFormat::Float3 =>     (3, gl::RGB32F, gl::RGB, gl::FLOAT),
			// 32x2
			TextureFormat::Float2 =>     (2, gl::RG16F, gl::RG, gl::FLOAT),
			// 16x4
			TextureFormat::Uint16x4 =>   (4, gl::RGBA16UI, gl::RGBA_INTEGER, gl::UNSIGNED_SHORT),
			TextureFormat::Sint16x4 =>   (4, gl::RGBA16I, gl::RGBA_INTEGER, gl::SHORT),
			TextureFormat::Unorm16x4 =>  (4, gl::RGBA16, gl::RGBA, gl::UNSIGNED_SHORT),
			TextureFormat::Snorm16x4 =>  (4, gl::RGBA16_SNORM, gl::RGBA, gl::SHORT),
			TextureFormat::Float16x4 =>  (4, gl::RGBA16F, gl::RGBA, gl::FLOAT),	// ??? 
			// 16x2
			TextureFormat::Uint16x2 =>   (2, gl::RG16UI, gl::RG_INTEGER, gl::UNSIGNED_SHORT),
			TextureFormat::Sint16x2 =>   (2, gl::RG16I, gl::RG_INTEGER, gl::SHORT),
			TextureFormat::Unorm16x2 =>  (2, gl::RG16, gl::RG, gl::UNSIGNED_SHORT),
			TextureFormat::Snorm16x2 =>  (2, gl::RG16_SNORM, gl::RG, gl::SHORT),
			TextureFormat::Float16x2 =>  (2, gl::RG16F, gl::RG, gl::FLOAT),	// ???
			// 8x4
			TextureFormat::Uint8x4 =>    (4, gl::RGBA8UI, gl::RGBA_INTEGER, gl::UNSIGNED_BYTE),
			TextureFormat::Sint8x4 =>    (4, gl::RGBA8I, gl::RGBA_INTEGER, gl::BYTE),
			TextureFormat::Unorm8x4 =>   (4, gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
			TextureFormat::Snorm8x4 =>   (4, gl::RGBA8_SNORM, gl::RGBA, gl::BYTE),
			// 8x3
			TextureFormat::Uint8x3 =>    (3, gl::RGB8UI, gl::RGB_INTEGER, gl::UNSIGNED_BYTE),
			TextureFormat::Sint8x3 =>    (3, gl::RGB8I, gl::RGB_INTEGER, gl::BYTE),
			TextureFormat::Unorm8x3 =>   (3, gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
			TextureFormat::Snorm8x3 =>   (3, gl::RGB8_SNORM, gl::RGB, gl::BYTE),
			// 8x2
			TextureFormat::Uint8x2 =>    (2, gl::RG8UI, gl::RG_INTEGER, gl::UNSIGNED_BYTE),
			TextureFormat::Sint8x2 =>    (2, gl::RG8I, gl::RG_INTEGER, gl::BYTE),
			TextureFormat::Unorm8x2 =>   (2, gl::RG8, gl::RG, gl::UNSIGNED_BYTE),
			TextureFormat::Snorm8x2 =>   (2, gl::RG8_SNORM, gl::RG, gl::BYTE),
			// 10_10_10_2
			TextureFormat::Unorm10x3_1x2 => (4, gl::RGB10_A2, gl::RGBA, gl::UNSIGNED_INT_10_10_10_2),
			TextureFormat::Snorm10x3_1x2 => (4, gl::RGB10_A2, gl::RGBA, gl::UNSIGNED_INT_10_10_10_2),
			// Single
			TextureFormat::Uint32 =>     (1, gl::R32UI, gl::RED, gl::UNSIGNED_INT),
			TextureFormat::Sint32 =>     (1, gl::R32I, gl::RED, gl::INT),
			TextureFormat::Uint16 =>     (1, gl::R16UI, gl::RED, gl::UNSIGNED_SHORT),
			TextureFormat::Sint16 =>     (1, gl::R16I, gl::RED, gl::SHORT),
			TextureFormat::Unorm16 =>    (1, gl::R16, gl::RED, gl::UNSIGNED_SHORT),
			TextureFormat::Snorm16 =>    (1, gl::R16_SNORM, gl::RED, gl::SHORT),
			//
			TextureFormat::Uint8 =>      (1, gl::R8UI, gl::RED, gl::UNSIGNED_BYTE),
			TextureFormat::Sint8 =>      (1, gl::R8I, gl::RED, gl::BYTE),
			TextureFormat::Unorm8 =>     (1, gl::R8, gl::RED, gl::UNSIGNED_BYTE),
			TextureFormat::Snorm8 =>     (1, gl::R8_SNORM, gl::RED, gl::BYTE),
			TextureFormat::Float16 =>    (1, gl::R16F, gl::RED, gl::HALF_FLOAT),
			TextureFormat::Float =>      (1, gl::R32F, gl::RED, gl::HALF_FLOAT)
		}
	}
}

pub struct Texture2D
{
	obj: GLuint,
	width: u32,
	height: u32,
	format: TextureFormat
}


impl Drop for Texture2D
{
	fn drop(&mut self)
	{
		unsafe {
			gl::DeleteTextures(1, &self.obj);
		}
	}
}

unsafe fn create_texture_2d<T>(
		width: u32, 
		height: u32, 
		num_mip_levels: u8, 
		format: TextureFormat, 
		initial_data: Option<&[T]>) -> GLuint
{
	let mut tex : GLuint = 0;
	let (num_elements, int_fmt, ext_fmt, ty) = format.gl_description();
	gl::GenTextures(1, &mut tex);
	gl::BindTexture(gl::TEXTURE_2D, tex);
	gl::TexStorage2D(gl::TEXTURE_2D, num_mip_levels as i32, int_fmt, width as i32, height as i32);
	if let Some(data) = initial_data {
		gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, width as i32, height as i32, ext_fmt, ty, data.as_ptr() as *const GLvoid);
	}
	gl::BindTexture(gl::TEXTURE_2D, 0);
	return tex;
}

impl Texture2D
{
	pub fn new<T>(
		width: u32, 
		height: u32, 
		num_mip_levels: u8, 
		format: TextureFormat, 
		initial_data: Option<&[T]>) -> Texture2D
	{

		let byte_size = mem::size_of::<T>() * (width * height * format.gl_description().0 as u32) as usize;
		trace!("{} x {}, {} mip levels, format: {:?}, byte_size: {}, initial_data byte size: {}", 
			width, height, num_mip_levels, format, byte_size, if let Some(data) = initial_data { data.len() * mem::size_of::<T>() } else {0});

		if let Some(data) = initial_data {
			assert!(byte_size == data.len() * mem::size_of::<T>());
		}

		unsafe 
		{
			Texture2D {
				obj: create_texture_2d(width, height, num_mip_levels, format, initial_data),
				width: width,
				height: height,
				format: format
			}
		}
	}

	pub fn bind(&self, unit: u32)
	{
		unsafe {
			gl::BindTextures(unit, 1, &self.obj);
		}
	}
}