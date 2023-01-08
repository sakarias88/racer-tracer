use std::option::Option;

use crate::geometry::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Sphere {
    pos: Vec3,
    radius: f64,
    material: Vec3, // Just a color for now.
}

impl Sphere {
    pub fn new(pos: Vec3, radius: f64, material: Vec3) -> Self {
        Self {
            pos,
            radius,
            material,
        }
    }
}

impl Clone for Sphere {
    fn clone(&self) -> Self {
        Self {
            pos: self.pos,
            radius: self.radius,
            material: self.material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin() - self.pos;
        let a = ray.direction().length_squared();
        let half_b = oc.dot(ray.direction());
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;

            if root < t_min || t_max < root {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = (point - self.pos) / self.radius;

        let mut hit_record = HitRecord::new(point, root, self.material);
        hit_record.set_face_normal(ray, outward_normal);
        Some(hit_record)
    }

    fn clone_box(&self) -> Box<dyn Hittable> {
        Box::new(self.clone())
    }
}
