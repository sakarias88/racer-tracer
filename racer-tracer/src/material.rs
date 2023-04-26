pub mod dialectric;
pub mod lambertian;
pub mod metal;

use crate::geometry::HitRecord;
use crate::ray::Ray;
use crate::vec3::Color;

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Ray, Color)>;
}
