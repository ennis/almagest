
use gl;
use gl::types::*;
use std::slice;
use libc::c_void;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;
use std::raw;
use super::attrib::InputLayout;

/// Treat a given slice as `&[u8]` for the given function call
pub fn as_byte_slice<T>(slice: &[T]) -> &[u8] {
    let len = mem::size_of::<T>() * slice.len();
    let slice = raw::Slice { data: slice.as_ptr(), len: len };
    unsafe {
        mem::transmute(slice)
    }
}

/// Treat a given object as `&[u8]` for the given function call
pub fn object_as_byte_slice<T>(obj: &T) -> &[u8] {
    let len = mem::size_of::<T>();
    let slice = raw::Slice { data: obj, len: len };
    unsafe {
        mem::transmute(slice)
    }
}


/// Buffer access by the CPU
#[derive(Copy, Clone, Debug)]
pub enum BufferAccess {
    // TODO: No immutable, unreadable by CPU
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Copy, Clone)]
pub enum BufferBindingHint {
    VertexBuffer,
    IndexBuffer,
    UniformBuffer,
}

#[derive(Copy, Clone)]
pub enum BufferUsage {
    Static,
    Dynamic,
    Stream,
}

#[derive(Copy, Clone)]
pub struct Binding<'a> {
    pub slot: u32,
    pub slice: RawBufSlice<'a>,
}

#[derive(Debug)]
pub struct RawBuffer<'a> {
    context: &'a BufferAllocator,
    access: BufferAccess,
    // XXX should only be public for the GL backend
    obj: GLuint,
    size: usize,
    map_ptr: *mut c_void,
}

impl<'a> RawBuffer<'a>
{
    pub fn as_raw_buf_slice(&self) -> RawBufSlice {
        RawBufSlice { raw: self, offset: 0, size: self.size }
    }

    pub unsafe fn as_buf_slice<T>(&'a self, offset: usize, num_elements: usize) -> BufSlice<'a, T> {
		// TODO check alignment of offset?
		// TODO check that offset + num_elements * mem::size_of::<T> < self.size
		// but since it's unsafe, might as well skip the checks
        BufSlice {
            raw: self,
            offset: offset,
            size: num_elements * mem::size_of::<T>(),
            _r: PhantomData,
        }
    }

    pub fn bind_as_element_array(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.obj);
        }
    }
}

// type-safe wrapper around a buffer object
pub struct Buffer<'a, T> {
    pub raw: RawBuffer<'a>,
    _r: PhantomData<T>,
}

#[derive(Copy, Clone, Debug)]
pub struct RawBufSlice<'a> {
    pub raw: &'a RawBuffer<'a>,
    pub offset: usize,
    pub size: usize,
}

// buffer slices
#[derive(Copy, Clone, Debug)]
pub struct BufSlice<'a, T> {
    pub raw: &'a RawBuffer<'a>,
    pub offset: usize,
    pub size: usize,
    _r: PhantomData<T>,
}

impl<'a, T> BufSlice<'a, T>
{
    // TODO: with this, we can have multiple overlapping writable
    // slices of the same buffer, which is not good
    pub fn as_raw<'b>(&'b self) -> RawBufSlice<'a> {
        RawBufSlice { raw: self.raw, offset: self.offset, size: self.size }
    }

    pub fn as_read_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.raw.map_ptr.offset(self.offset as isize) as *const T,
                                  self.size / mem::size_of::<T>())
        }
    }

    pub fn as_write_slice(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.raw.map_ptr.offset(self.offset as isize) as *mut T,
                                      self.size / mem::size_of::<T>())
        }
    }

    pub fn as_rw_slice(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.raw.map_ptr.offset(self.offset as isize) as *mut T,
                                      self.size / mem::size_of::<T>())
        }
    }
}

#[derive(Debug)]
pub struct BufferAllocator;


impl<'a, T> Buffer<'a, T>
{
	// TODO check access flags
    fn get_read_mapping(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.raw.map_ptr as *const T, self.raw.size / mem::size_of::<T>())
        }
    }

	// TODO check access flags
	// allow only one write mapping at a time
    fn get_write_mapping(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.raw.map_ptr as *mut T,
                                      self.raw.size / mem::size_of::<T>())
        }
    }

	// TODO check access flags
    fn get_rw_mapping(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.raw.map_ptr as *mut T,
                                      self.raw.size / mem::size_of::<T>())
        }
    }
}

impl<'a> Drop for RawBuffer<'a>
{
    fn drop(&mut self) {
        unsafe {
			//trace!("Deleting buffer {}", self.obj);
            gl::DeleteBuffers(1, &mut self.obj)
        }
    }
}

const NUM_POOLS: u32 = 9;
const MIN_BLOCK_SIZE_LOG: u32 = 8;
const MIN_BLOCK_SIZE: usize = 1 << MIN_BLOCK_SIZE_LOG;
const MAX_BLOCK_SIZE: usize = MIN_BLOCK_SIZE << (NUM_POOLS - 1);
const POOL_PAGE_SIZE: usize = 1024 * 1024;

fn get_gl_usage_flags(usage: BufferUsage) -> u32 {
	// TODO other usage scenarios? (READ, COPY)
    match usage
	{
		BufferUsage::Static => gl::STATIC_DRAW,
		BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
		BufferUsage::Stream => gl::STREAM_DRAW
	}
}

fn get_gl_access_flags(access: BufferAccess) -> u32 {
    match access
	{
		BufferAccess::ReadOnly => gl::MAP_READ_BIT,
		BufferAccess::WriteOnly => gl::MAP_WRITE_BIT,
		BufferAccess::ReadWrite => gl::MAP_READ_BIT | gl::MAP_WRITE_BIT
	}
}

fn get_gl_storage_flags(access: BufferAccess, usage: BufferUsage) -> u32 {
    let access_bits = match access
	{
		BufferAccess::ReadOnly => gl::MAP_READ_BIT,
		BufferAccess::WriteOnly => gl::MAP_WRITE_BIT,
		BufferAccess::ReadWrite => gl::MAP_READ_BIT | gl::MAP_WRITE_BIT
	};

    let usage_bits = match usage
	{
		BufferUsage::Static => 0,
		BufferUsage::Dynamic => gl::DYNAMIC_STORAGE_BIT,
		BufferUsage::Stream => 0
	};

    access_bits | usage_bits
}


fn get_gl_binding(binding: BufferBindingHint) -> u32 {
    match binding
	{
		BufferBindingHint::VertexBuffer => gl::ARRAY_BUFFER,
		BufferBindingHint::IndexBuffer => gl::ELEMENT_ARRAY_BUFFER,
		BufferBindingHint::UniformBuffer => gl::UNIFORM_BUFFER
	}
}

impl BufferAllocator
{
    pub fn alloc_raw_buffer<'a>(&'a self,
                                byte_size: usize,
                                access: BufferAccess,
                                binding: BufferBindingHint,
                                usage: BufferUsage,
                                initial_data: Option<&[u8]>)
                                -> RawBuffer<'a> {
        let mut obj: GLuint = 0;
        let ptr: *mut c_void;
        if let Some(d) = initial_data {
            assert!(byte_size == d.len());
        }
        unsafe {
            let binding_gl = get_gl_binding(binding);
            let map_flags = get_gl_access_flags(access) | gl::MAP_PERSISTENT_BIT |
                            gl::MAP_COHERENT_BIT | gl::MAP_INVALIDATE_BUFFER_BIT/* |
				gl::MAP_UNSYNCHRONIZED_BIT*/;
            let storage_flags = get_gl_storage_flags(access, usage) /*|
				gl::MAP_PERSISTENT_BIT |
				gl::MAP_COHERENT_BIT*/;
            gl::GenBuffers(1, &mut obj);
            gl::BindBuffer(binding_gl, obj);
            gl::BufferStorage(binding_gl,
                              byte_size as i64,
                              if let Some(d) = initial_data {
                    d.as_ptr() as *const GLvoid
                } else {
                    0 as *const GLvoid
                },
                              storage_flags);
            ptr = gl::MapBufferRange(
				binding_gl,
				0, byte_size as i64,
				map_flags);
        }
        RawBuffer {
            context: self,
            access: BufferAccess::ReadWrite,
            obj: obj,
            size: byte_size,
            map_ptr: ptr,
        }
    }

    pub fn alloc_buffer<'a, T>(&'a self,
                               num_elements: usize,
                               access: BufferAccess,
                               binding: BufferBindingHint,
                               usage: BufferUsage,
                               initial_data: Option<&[T]>)
                               -> Buffer<'a, T> {
        let byte_size = mem::size_of::<T>() * num_elements;
        Buffer {
            raw: self.alloc_raw_buffer(byte_size, access, binding, usage,
				if let Some(slice) = initial_data { Some(as_byte_slice(slice)) } else { None }),
            _r: PhantomData,
        }
    }

}

pub fn bind_vertex_buffers(layout: &InputLayout, vertex_buffers: &[RawBufSlice]) {
    unsafe {
        gl::BindVertexArray(layout.vao);
        let vbs : Vec<_> = vertex_buffers.iter().map(|&b| b.raw.obj).collect();
        let offsets : Vec<_> =  vertex_buffers.iter().map(|&b| b.offset as i64).collect();
        let strides : &[i32] = &layout.strides[..];
        gl::BindVertexBuffers(0, 1, vbs.as_ptr(), offsets.as_ptr(), strides.as_ptr());
    }
}

pub fn bind_index_buffer(index_buffer: &RawBufSlice) {
    unsafe {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer.raw.obj);
    }
}

pub fn bind_uniform_buffers(uniform_buffers: &[Binding]) {
    for binding in uniform_buffers {
        unsafe {
            gl::BindBufferRange(gl::UNIFORM_BUFFER,
                                binding.slot,
                                binding.slice.raw.obj,
                                binding.slice.offset as i64,
                                binding.slice.size as i64);
        }
    }
}
