//! This crate contains client-only components of Playform.

#![deny(missing_docs)]

extern crate bincode;
extern crate cgmath;
extern crate collision;
extern crate common;
extern crate fnv;
extern crate gl;
extern crate hound;
extern crate image;
extern crate isosurface_extraction;
#[macro_use]
extern crate log;
extern crate libc;
extern crate num;
#[cfg(feature="audio")]
extern crate portaudio;
extern crate rand;
extern crate stopwatch;
extern crate thread_scoped;
extern crate time;
extern crate voxel_data;
extern crate yaglw;
#[macro_use]
extern crate glium;

pub mod audio;
pub mod audio_loader;
pub mod audio_thread;
pub mod chunk;
pub mod chunk_stats;
pub mod client;
pub mod hud;
pub mod lod;
pub mod process_event;
pub mod record_book;
pub mod run;
pub mod server;
pub mod server_update;
pub mod terrain;
pub mod terrain_mesh;
pub mod update_thread;
pub mod vertex;
pub mod view;

pub use run::run;
