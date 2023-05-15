use std::sync::Arc;

use crate::{
    texture::{solid_color::SolidColor, Texture},
    vec3::{Color, Vec3},
};

use super::Material;

pub struct DiffuseLight {
    texture: Arc<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

    pub fn new_with_color(color: Color) -> Self {
        Self::new(Arc::new(SolidColor::new(color)))
    }
}

impl Material for DiffuseLight {
    fn scatter(
        &self,
        _ray: &crate::ray::Ray,
        _hit_record: &crate::geometry::HitRecord,
    ) -> Option<(crate::ray::Ray, Vec3)> {
        None
    }

    fn color_emitted(&self, u: f64, v: f64, point: &Vec3) -> Color {
        self.texture.value(u, v, point)
    }
}
