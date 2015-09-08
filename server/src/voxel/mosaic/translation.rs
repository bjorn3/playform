use cgmath::{Point, Point3, Vector3};

use voxel::{field, mosaic};

#[derive(Debug, Clone, Copy)]
pub struct T<Mosaic> {
  pub translation: Vector3<f32>,
  pub mosaic: Mosaic,
}

impl<Mosaic> field::T for T<Mosaic> where Mosaic: field::T {
  fn density(&self, p: &Point3<f32>) -> f32 {
    let p = p.add_v(&-self.translation);
    field::T::density(&self.mosaic, &p)
  }

  fn normal(&self, p: &Point3<f32>) -> Vector3<f32> {
    let p = p.add_v(&-self.translation);
    field::T::normal(&self.mosaic, &p)
  }
}

impl<Mosaic> mosaic::T for T<Mosaic> where Mosaic: mosaic::T {
  fn density(&self, p: &Point3<f32>) -> f32 {
    let p = p.add_v(&-self.translation);
    mosaic::T::density(&self.mosaic, &p)
  }

  fn material(&self, p: &Point3<f32>) -> Option<::voxel::Material> {
    let p = p.add_v(&-self.translation);
    mosaic::T::material(&self.mosaic, &p)
  }
}