use crate::{
    material::Material,
    ray::Ray,
    util::random_double,
    vec3::{dot, reflect, refract, Color},
};

pub struct Dialectric {
    refraction_index: f64,
}

impl Dialectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        // Schlick approximation
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
    }
}

impl Material for Dialectric {
    fn scatter(
        &self,
        ray: &crate::ray::Ray,
        rec: &crate::geometry::HitRecord,
    ) -> Option<(Ray, Color)> {
        let refraction_ratio = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = &ray.direction().unit_vector();
        let cos_theta = f64::min(dot(&-unit_direction, &rec.normal), 1.0);
        let sin_theta = f64::sqrt(1.0 - cos_theta * cos_theta);

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction = if cannot_refract
            || Dialectric::reflectance(cos_theta, refraction_ratio) > random_double()
        {
            reflect(unit_direction, &rec.normal)
        } else {
            refract(unit_direction, &rec.normal, refraction_ratio)
        };

        Some((
            Ray::new(rec.point, direction, ray.time()),
            Color::new(1.0, 1.0, 1.0),
        ))
    }
}
