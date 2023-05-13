pub mod checkered;
pub mod image;
pub mod noise;
pub mod solid_color;

use crate::vec3::{Color, Vec3};

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, point: &Vec3) -> Color;
}
