pub mod r#box;
pub mod moving_sphere;
pub mod rotate_y;
pub mod sphere;
pub mod translate;
pub mod xy_rect;
pub mod xz_rect;
pub mod yz_rect;

use std::sync::Arc;

use crate::aabb::Aabb;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub material: Arc<dyn Material>,
    pub u: f64,
    pub v: f64,
    pub obj_id: usize,
}

impl HitRecord {
    fn new(
        point: Vec3,
        t: f64,
        material: Arc<dyn Material>,
        u: f64,
        v: f64,
        obj_id: usize,
    ) -> Self {
        Self {
            point,
            normal: Vec3::default(),
            t,
            front_face: true,
            material,
            u,
            v,
            obj_id,
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
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;

    fn bounding_box(&self, time_a: f64, time_b: f64) -> &Aabb;
}
