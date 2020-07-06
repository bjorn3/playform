//! Data structures and functions for dealing with lighting.

use cgmath::Vector3;
use common::color::Color3;
use gl;
use std;
use glium::uniforms::{Uniforms, EmptyUniforms, UniformsStorage, AsUniformValue};

#[derive(Debug, Clone)]
/// Colored sun data structure.
pub struct Sun {
  /// How far through the day the sun is, in [0, 1).
  pub progression : f32,
  /// The rotation of the sun's path about the y axis.
  pub rotation: f32,
}

impl Sun {
  fn sin_cos(&self) -> (f32, f32) {
    // Convert to radians.
    let angle = self.progression * 2.0 * std::f32::consts::PI;
    angle.sin_cos()
  }

  /// A vector pointing toward the sun.
  pub fn direction(&self) -> Vector3<f32> {
    let (s, c) = self.sin_cos();
    Vector3::new(c, s, 0.0)
  }

  /// The intensity of direct lighting from the sun.
  pub fn intensity(&self) -> Color3<f32> {
    let (s, c) = self.sin_cos();
    Color3::of_rgb(
      c.abs(),
      (s + 1.0) / 2.0,
      (s * 0.75 + 0.25).abs(),
    )
  }

  /// The intensity of ambient lighting from the sun.
  pub fn ambient_intensity(&self) -> Color3<f32> {
    let (s, _) = self.sin_cos();
    let ambient_light = f32::max(0.4, s / 2.0);
    let i = self.intensity();
    Color3::of_rgb(
      i.r * ambient_light,
      i.g * ambient_light,
      i.b * ambient_light,
    )
  }
}

/// Sets the `sun` struct in some shader.
pub fn set_sun<'n, T: AsUniformValue, R: Uniforms>(
  uniforms: UniformsStorage<'n, T, R>,
  sun: &Sun,
) -> UniformsStorage<'n, [f32; 3], UniformsStorage<'n, [f32; 3], UniformsStorage<'n, T, R>>>{
  let d = sun.direction();
  let i = sun.intensity();
  uniforms
    .set("sun.direction", [d.x, d.y, d.z])
    .set("sun.intensity", [i.r, i.g, i.b])
}

/// Sets the `ambient_light` uniform in some shader.
pub fn set_ambient_light<'n, T: AsUniformValue, R: Uniforms>(
  uniforms: UniformsStorage<'n, T, R>,
  sun: &Sun,
) -> UniformsStorage<'n, [f32; 3], UniformsStorage<'n, T, R>>{
  let a = sun.ambient_intensity();
  uniforms.set("ambient_light", [a.r, a.g, a.b])
}
