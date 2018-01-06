//! This module contains the game's custom shader structs.

use cgmath;
use cgmath::{Vector2};
use gl;
use std;
use glium::{Display, Program};

use view::camera;

/// Load a shader from a filename prefix.
/// e.g. shader_from_prefix("foo") will load a vertex shader from shaders/foo.vs.glsl and fragment shader
/// from shaders/foo.fs.glsl.
pub fn shader_from_prefix(gl: &Display, prefix: &'static str) -> Program {
  let read_preprocessed_shader =
    |name| {
      String::from_utf8(
        std::process::Command::new("m4")
        .arg(name)
        .current_dir(std::path::Path::new("shaders/"))
        .output()
        .unwrap()
        .stdout
      ).unwrap()
    };
  let vs = read_preprocessed_shader(format!("{}.vs.glsl", prefix));
  let fs = read_preprocessed_shader(format!("{}.fs.glsl", prefix));
  debug!("loaded {} vertex shader:", prefix);
  debug!("{}", vs);
  debug!("loaded {} fragment shader:", prefix);
  debug!("{}", fs);
  Program::from_source(gl, &vs, &fs, None).unwrap()
}

/// The game's custom shader structs.
#[allow(missing_docs)]
pub struct T {
  pub mob_shader: Program,
  pub terrain_shader: Program,
  pub texture_shader: Program,
  pub grass_billboard: Program,
  pub hud_color_shader: Program,
  pub sky: Program,
}

#[allow(missing_docs)]
pub fn new(gl: &mut Display, window_size: Vector2<i32>) -> T {
  let terrain_shader       = terrain_program(gl);
  let mob_shader           = color_program(gl);
  let mut hud_color_shader = color_program(gl);
  let texture_shader       = texture_program(gl);
  let grass_billboard      = grass_billboard_program(gl);
  let sky                  = sky_program(gl);

  let hud_camera = {
    let mut c = camera::unit();
    let dx = window_size.x as f32 / window_size.y as f32;
    let dy = 1.0;
    c.fov = cgmath::ortho(-dx, dx, -dy, dy, -1.0, 1.0);
    c
  };

  camera::set_camera(
    &mut hud_color_shader.shader,
    gl,
    &hud_camera,
  );

  T {
    mob_shader: mob_shader,
    terrain_shader: terrain_shader,
    texture_shader: texture_shader,
    grass_billboard: grass_billboard,
    hud_color_shader: hud_color_shader,
    sky: sky,
  }
}

/// Draw linearly-interpolated colored vertices in 3D space.
pub fn grass_billboard_program(gl: &Display) -> Program {
  shader_from_prefix(gl, "grass_billboard")
}

/// Draw linearly-interpolated colored vertices in 3D space.
pub fn color_program(gl: &Display) -> Program {
  shader_from_prefix(gl, "color")
}

/// Draw linearly-interpolated colored vertices in 3D space.
pub fn sky_program(gl: &Display) -> Program {
  shader_from_prefix(gl, "sky")
}

/// Draw linearly-interpolated colored vertices in 3D space.
pub fn terrain_program(gl: &Display) -> Program {
  shader_from_prefix(gl, "terrain")
}

/// Draw linearly-interpolated colored vertices in 3D space.
pub fn texture_program(gl: &Display) -> Program {
  shader_from_prefix(gl, "texture")
}
