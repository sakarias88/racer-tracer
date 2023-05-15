use std::sync::Arc;

use crate::vec3::{Color, Vec3};

use super::{solid_color::SolidColor, Texture};

pub struct Checkered {
    even: Arc<dyn Texture>,
    odd: Arc<dyn Texture>,
    checker_size: f64,
}

impl Checkered {
    pub fn new(even: Arc<dyn Texture>, odd: Arc<dyn Texture>) -> Self {
        Self {
            even,
            odd,
            checker_size: 10.0,
        }
    }

    #[allow(dead_code)]
    pub fn new_with_colors(even: Color, odd: Color) -> Self {
        Self {
            even: Arc::new(SolidColor::new(even)),
            odd: Arc::new(SolidColor::new(odd)),
            checker_size: 10.0,
        }
    }
}

impl Texture for Checkered {
    fn value(&self, u: f64, v: f64, point: &Vec3) -> Color {
        let sines = (point.x() * self.checker_size).sin()
            * (point.y() * self.checker_size).sin()
            * (point.z() * self.checker_size).sin();
        if sines < 0.0 {
            self.odd.value(u, v, point)
        } else {
            self.even.value(u, v, point)
        }
    }
}
