pub mod sphere;

use std::sync::Arc;

use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub material: Arc<Box<dyn Material>>,
}

impl HitRecord {
    fn new(point: Vec3, t: f64, material: Arc<Box<dyn Material>>) -> Self {
        Self {
            point,
            normal: Vec3::default(),
            t,
            front_face: true,
            material,
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

pub trait Hittable: Send + Sync {
    //pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}
