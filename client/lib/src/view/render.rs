//! Draw the view.

use cgmath;
use gl;
use time;
use yaglw;
use glium::{self, Frame, Surface};
use glium::index::{NoIndices, PrimitiveType, VertexBuffer};
use glium::uniforms::{Uniforms, EmptyUniforms, UniformsStorage, AsUniformValue};

use view;
use view::camera::set_camera;
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
  params: &glium::DrawParameters,
  frame: &mut Frame,
) {
  let uniforms = uniform! {
    time_ms: (time::precise_time_ns() / 1_000_000) as f32,
    alpha_threshold: 0.5,
    //texture_in: rndr.grass_texture
  };
  let uniforms = set_ambient_light(uniforms, &rndr.sun);
  let uniforms = set_camera(uniforms, &rndr.camera);
  let uniforms = set_clip(uniforms, rndr.near_clip, rndr.far_clip);
  let uniforms = set_eye_position(uniforms, &rndr.camera);
  let uniforms = set_sun(uniforms, &rndr.sun);
  frame.draw(
    rndr.grass_buffers,
    NoIndices(PrimitiveType::TrianglesList),
    &rndr.shaders.grass_billboard_shader,
    &uniforms,
    &params
  );
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
  let terrain_uniforms = set_ambient_light(terrain_uniforms, &rndr.sun);
  let terrain_uniforms = set_camera(terrain_uniforms, &rndr.camera);
  let terrain_uniforms = set_clip(terrain_uniforms, rndr.near_clip, rndr.far_clip);
  let terrain_uniforms = set_eye_position(terrain_uniforms, &rndr.camera);
  let terrain_uniforms = set_sun(terrain_uniforms, &rndr.sun);
  frame.draw(
    rndr.terrain_buffers,
    NoIndices(PrimitiveType::TrianglesList),
    &rndr.shaders.terrain_shader,
    &terrain_uniforms,
    &params
  );

  let player_uniforms = uniform! {
    __dummy: 0
  };
  let player_uniforms = set_camera(player_uniforms, &rndr.camera);
  let terrain_uniforms = set_clip(terrain_uniforms, rndr.near_clip, rndr.far_clip);
  frame.draw(
    rndr.mob_buffers,
    NoIndices(PrimitiveType::TrianglesList),
    &rndr.shaders.mob_shader,
    &player_uniforms,
    &params
  );
  frame.draw(
    rndr.player_buffers,
    NoIndices(PrimitiveType::TrianglesList),
    &rndr.shaders.mob_shader,
    &player_uniforms,
    &params
  );

  draw_grass_billboards(rndr, &params, frame);

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
