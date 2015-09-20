use gl;
use gl::types::*;

#[derive(Copy, Clone, Debug)]
pub enum TextureAddressMode
{
    Clamp,
    Mirror,
    Wrap
}

impl TextureAddressMode
{
    pub fn to_gl(self) -> u32
	{
		match self {
			TextureAddressMode::Clamp => gl::CLAMP_TO_EDGE,
			TextureAddressMode::Mirror => gl::MIRRORED_REPEAT,
			TextureAddressMode::Wrap => gl::REPEAT
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum TextureMinFilter
{
    Nearest,
    Linear
}

impl TextureMinFilter
{
    pub fn to_gl(self) -> u32
    {
        match self
        {
            TextureMinFilter::Nearest => gl::NEAREST,
            TextureMinFilter::Linear => gl::LINEAR
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TextureMagFilter
{
    Nearest,
    Linear
}

impl TextureMagFilter
{
    pub fn to_gl(self) -> u32
    {
        match self
        {
            TextureMagFilter::Nearest => gl::NEAREST,
            TextureMagFilter::Linear => gl::LINEAR
        }
    }
}

// 2D sampler
#[derive(Clone)]
pub struct Sampler2DDesc
{
    pub addr_u: TextureAddressMode,
    pub addr_v: TextureAddressMode,
    pub min_filter: TextureMinFilter,
    pub mag_filter: TextureMagFilter,
}

pub struct Sampler2D
{
    desc: Sampler2DDesc,
    obj: GLuint
}

impl Sampler2D
{
    pub fn bind(&self, texunit: u32)
    {
        unsafe {
            gl::BindSampler(self.obj, texunit);
        }
    }
}


impl Sampler2DDesc
{
    pub fn default() -> Sampler2DDesc
    {
        Sampler2DDesc {
            addr_u: TextureAddressMode::Clamp,
            addr_v: TextureAddressMode::Clamp,
            min_filter: TextureMinFilter::Nearest,
            mag_filter: TextureMagFilter::Linear
        }
    }

    pub fn build(&self) -> Sampler2D
    {
        let mut sampler : GLuint = 0;
        unsafe {
            gl::GenSamplers(1, &mut sampler);
            gl::SamplerParameteri(sampler, gl::TEXTURE_MIN_FILTER, self.min_filter.to_gl() as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER, self.mag_filter.to_gl() as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }

        Sampler2D {
            desc: self.clone(),
            obj: sampler
        }
    }
}
