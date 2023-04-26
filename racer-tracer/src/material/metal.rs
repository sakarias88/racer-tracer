use crate::{
    material::Material,
    ray::Ray,
    vec3::{random_in_unit_sphere, reflect, Color},
};

pub struct Metal {
    color: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(color: Color, fuzz: f64) -> Self {
        Self { color, fuzz }
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
            Some((scattered, self.color))
        }
    }
}
