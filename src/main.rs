#![feature(raw)]
#![feature(associated_consts)]
#![allow(dead_code, unused_imports, unused_variables, non_camel_case_types)]
extern crate nalgebra;
extern crate glfw;
extern crate gl;
extern crate libc;
#[macro_use]
extern crate log;
extern crate fern;
extern crate time;
extern crate num;
extern crate typed_arena;
extern crate image;
extern crate smallvec;

mod scene;
mod rendering;
mod glutil;
mod context;
mod buffer;
mod frame;
mod shader;
mod attrib;
mod draw;
mod texture;
mod render_queue;
mod camera;
mod input;

mod sample_scene;

fn main()
{
	sample_scene::sample_scene();
}