pub mod sphere;

use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub color: Vec3,
}

impl HitRecord {
    fn new(point: Vec3, t: f64, color: Vec3) -> Self {
        Self {
            point,
            normal: Vec3::default(),
            t,
            front_face: true,
            color,
        }
    }

    fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = ray.direction().dot(&outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

impl Default for HitRecord {
    fn default() -> Self {
        HitRecord {
            point: Vec3::default(),
            normal: Vec3::default(),
            t: 0.0,
            front_face: true,
            color: Vec3::default(),
        }
    }
}

pub trait Hittable {
    //pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}
