use crate::{
    material::Material,
    ray::Ray,
    vec3::{random_unit_vector, Color, Vec3},
};

pub struct Lambertian {
    color: Color,
}

impl Lambertian {
    pub fn new(color: Color) -> Self {
        Self { color }
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

        let scattered = Ray::new(rec.point, scatter_direction);
        Some((scattered, self.color))
    }
}
