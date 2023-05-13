use std::sync::Arc;

use crate::{
    material::Material,
    ray::Ray,
    texture::{solid_color::SolidColor, Texture},
    vec3::{random_unit_vector, Color},
};

pub struct Lambertian {
    texture: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

    pub fn new_with_color(color: Color) -> Self {
        Self::new(Arc::new(SolidColor::new(color)))
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        ray: &crate::ray::Ray,
        rec: &crate::geometry::HitRecord,
    ) -> Option<(Ray, Color)> {
        let mut scatter_direction = rec.normal + random_unit_vector();

        // Catch bad scatter direction
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((
            Ray::new(rec.point, scatter_direction, ray.time()),
            self.texture.value(rec.u, rec.v, &rec.point),
        ))
    }
}
