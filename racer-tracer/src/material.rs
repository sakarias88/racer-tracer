pub mod dialectric;
pub mod lambertian;
pub mod metal;

use std::sync::Arc;

use crate::geometry::HitRecord;
use crate::ray::Ray;
use crate::vec3::Color;

pub type SharedMaterial = Arc<Box<dyn Material>>;

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Ray, Color)>;
}
