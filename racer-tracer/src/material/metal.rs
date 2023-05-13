use std::sync::Arc;

use crate::{
    material::Material,
    ray::Ray,
    texture::{solid_color::SolidColor, Texture},
    vec3::{random_in_unit_sphere, reflect, Color},
};

pub struct Metal {
    texture: Arc<dyn Texture>,
    fuzz: f64,
}

impl Metal {
    pub fn new(texture: Arc<dyn Texture>, fuzz: f64) -> Self {
        Self { texture, fuzz }
    }

    pub fn new_with_color(color: Color, fuzz: f64) -> Self {
        Self::new(Arc::new(SolidColor::new(color)), fuzz)
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        ray: &crate::ray::Ray,
        rec: &crate::geometry::HitRecord,
    ) -> Option<(Ray, Color)> {
        let reflected = reflect(&ray.direction().unit_vector(), &rec.normal);
        let scattered = Ray::new(
            rec.point,
            reflected + self.fuzz * random_in_unit_sphere(),
            ray.time(),
        );

        if scattered.direction().dot(&rec.normal) < 0.0 {
            None
        } else {
            Some((scattered, self.texture.value(rec.u, rec.v, &rec.point)))
        }
    }
}
