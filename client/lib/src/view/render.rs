//! Draw the view.

use cgmath;
use gl;
use time;
use yaglw;
use glium::{self, Frame, Surface};
use glium::index::{NoIndices, PrimitiveType, VertexBuffer};
use glium::uniforms::{Uniforms, EmptyUniforms, UniformsStorage, AsUniformValue};

use view;
use view::camera::{set_camera};
use view::light::{set_sun, set_ambient_light};

fn set_eye_position<'n, T: AsUniformValue, R: Uniforms>(
  uniforms: UniformsStorage<'n, T, R>,
  camera: &view::camera::T,
) -> UniformsStorage<'n, [f32; 3], UniformsStorage<'n, T, R>>{
  uniforms.add("eye_position", [camera.position.x, camera.position.y, camera.position.z])
}

fn set_clip<'n, T: AsUniformValue, R: Uniforms>(
  uniforms: UniformsStorage<'n, T, R>,
  near: f32,
  far: f32,
) -> UniformsStorage<'n, f32, UniformsStorage<'n, f32, UniformsStrage<'n, T, R>>>{
  uniforms.add("near_clip", near).add("far_clip", far)
}

fn set_clip(shader: &mut yaglw::shader::Shader, near: f32, far: f32) {
  unsafe {
    let uniform = shader.get_uniform_location("near_clip");
    gl::Uniform1f(uniform, near);
    let uniform = shader.get_uniform_location("far_clip");
    gl::Uniform1f(uniform, far);
  }
}

fn draw_backdrop(
  rndr: &mut view::T,
  params: &glium::DrawParameters,
  frame: &mut Frame,
) {
  let sun_direction = sun.direction();
  let uniforms = uniform! {
    time_ms: (time::precise_time_ns() / 1_000_000) as f32,
    projection_matrix: rndr.camera.projection_matrix(),
    window_size: [rndr.window_size.x as f32, rndr.window_size.y as f32],
    sun_direction: [sun_direction.x, sun_direction.y, sun_direction.z],
    sun_intensity: [1.0, 1.0, 1.0],
  };
  let uniforms = set_eye_position(uniforms, &rndr.camera);
  frame.draw(
    VertexBuffer::new(frame, &[]),
    NoIndices(PrimitiveType::TriangleStrip),
    &rndr.shaders.sky,
    &EmptyUniforms,
    params
  );
  frame.clear_depth(-1.0);
}

fn draw_grass_billboards(
  rndr: &mut view::T,
) {
  rndr.shaders.grass_billboard.shader.use_shader(&mut rndr.gl);
  unsafe {
    let time_ms_uniform = rndr.shaders.grass_billboard.shader.get_uniform_location("time_ms");
    gl::Uniform1f(time_ms_uniform, (time::precise_time_ns() / 1_000_000) as f32);
  }
  let uniforms = uniform! {
    __dummy: 0
  };
  set_ambient_light(&mut rndr.shaders.grass_billboard.shader, &mut rndr.gl, &rndr.sun);
  set_camera(&mut rndr.shaders.grass_billboard.shader, &mut rndr.gl, &rndr.camera);
  let uniforms = set_clip(uniforms, rndr.near_clip, rndr.far_clip);
  let uniforms = set_eye_position(uniforms, &rndr.camera);
  set_sun(&mut rndr.shaders.grass_billboard.shader, &mut rndr.gl, &rndr.sun);
  let alpha_threshold_uniform =
    rndr.shaders.grass_billboard.shader.get_uniform_location("alpha_threshold");
  unsafe {
    gl::Disable(gl::CULL_FACE);
    gl::Uniform1f(alpha_threshold_uniform, 0.5);
    gl::ActiveTexture(rndr.misc_texture_unit.gl_id());
    gl::BindTexture(gl::TEXTURE_2D, rndr.grass_texture.handle.gl_id);
  }
  rndr.grass_buffers.draw(&mut rndr.gl);
}

#[allow(missing_docs)]
pub fn render(
  rndr: &mut view::T,
) {
  let frame = rndr.gl.draw();
  frame.clear(None, Some((0., 0., 0., 0.)), false, None, None);

  let params = glium::DrawParameters {
    depth: glium::Depth {
      test: glium::DepthTest::IfLess,
      write: true,
      .. Default::default()
    },
    blend: glium::Blend {
      color: glium::BlendingFunction::Addition {
        source: glium::LinearBlendingFactor::SourceAlpha,
        destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
      },
      .. Default::default()
    },
    line_width: Some(2.5),
    backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
    smooth: Some(glium::Smooth::Nicest),
    .. Default::default()
  };

  draw_backdrop(rndr, &params, frame);

  // draw the world
  let terrain_uniforms = uniform! {
    __dummy: 0
  };
  rndr.shaders.terrain_shader.shader.use_shader(&mut frame);
  set_ambient_light(&mut rndr.shaders.terrain_shader.shader, &mut frame, &rndr.sun);
  set_camera(&mut rndr.shaders.terrain_shader.shader, &mut frame, &rndr.camera);
  let terrain_uniforms = set_clip(terrain_uniforms, rndr.near_clip, rndr.far_clip);
  let terrain_uniforms = set_eye_position(terrain_uniforms, &rndr.camera);
  set_sun(&mut rndr.shaders.terrain_shader.shader, &mut frame, &rndr.sun);
  rndr.terrain_buffers.draw(&mut frame);

  rndr.shaders.mob_shader.use_shader(&mut frame);
  set_camera(&mut rndr.shaders.mob_shader, &mut frame, &rndr.camera);
  let terrain_uniforms = set_clip(terrain_uniforms, rndr.near_clip, rndr.far_clip);
  rndr.mob_buffers.draw(&mut frame);
  rndr.player_buffers.draw(&mut frame);

  draw_grass_billboards(rndr);

  if rndr.show_hud {
    frame.draw(
      rndr.hud_triangles,
      NoIndices(PrimitiveType::TrianglesList),
      &rndr.shaders.hud_color_shader,
      &EmptyUniforms,
      &params
    );
  }

  frame.finish().unwrap();
}
