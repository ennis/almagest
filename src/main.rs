#![feature(raw)]
#![feature(associated_consts)]
#![feature(custom_derive, plugin)]
// TODO remove this
#![feature(vec_push_all)]
#![plugin(serde_macros)]
#![plugin(peg_syntax_ext)]
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
extern crate tobj;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate bitflags;
extern crate combine;

mod scene;
mod rendering;
mod camera;
mod event;
mod window;
mod material;
mod scene_data;
mod terrain;
mod asset_loader;
mod shadow_pass;
mod graphics;
mod player;

mod sample_scene;

fn main()
{
	sample_scene::sample_scene();
}
