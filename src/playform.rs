//! The entry point.
#![crate_type = "bin"]
#![deny(warnings)]
#![deny(missing_doc)]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(unsafe_destructor)]
#![feature(phase)]

extern crate gl;
extern crate glw;
extern crate event;
extern crate input;
extern crate libc;
#[phase(plugin, link)]
extern crate log;
extern crate nalgebra;
extern crate ncollide;
extern crate png;
extern crate sdl2;
extern crate sdl2_game_window;
extern crate shader_version;

// so unwrap! is defined for mods below
mod common;
mod block;
mod borrow;
mod fontloader;
mod id_allocator;
mod player;
// so time! macro is defined in main
mod stopwatch;
mod loader;
mod main;
mod mob;
mod octree;
mod physics;
mod shader;
mod ttf;

#[allow(dead_code)]
fn main() {
  return main::main();
}
