pub mod dialectric;
pub mod diffuse_light;
pub mod lambertian;
pub mod metal;

use crate::geometry::HitRecord;
use crate::ray::Ray;
use crate::vec3::{Color, Vec3};

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Ray, Color)>;
    fn color_emitted(&self, _u: f64, _v: f64, _point: &Vec3) -> Color {
        Color::default()
    }
}
