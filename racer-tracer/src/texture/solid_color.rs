use crate::vec3::{Color, Vec3};

use super::Texture;

pub struct SolidColor {
    color: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        Self { color }
    }

    pub fn new_from_rgb(r: f64, g: f64, b: f64) -> Self {
        Self::new(Vec3::new(r, g, b))
    }

    #[allow(dead_code)]
    pub fn color(&self) -> &Color {
        &self.color
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _point: &Vec3) -> Color {
        self.color
    }
}
