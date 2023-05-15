use crate::{ray::Ray, vec3::Color};

pub trait BackgroundColor: Send + Sync {
    fn color(&self, ray: &Ray) -> Color;
}

pub struct Sky {
    top: Color,
    bottom: Color,
}

impl Sky {
    pub fn new(top: Color, bottom: Color) -> Self {
        Self { top, bottom }
    }
}

impl Default for Sky {
    fn default() -> Self {
        Self {
            top: Color::new(1.0, 1.0, 1.0),
            bottom: Color::new(0.5, 0.7, 1.0),
        }
    }
}

impl BackgroundColor for Sky {
    fn color(&self, ray: &Ray) -> Color {
        let unit_direction = ray.direction().unit_vector();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * self.top + t * self.bottom
    }
}

pub struct SolidBackgroundColor {
    color: Color,
}

impl SolidBackgroundColor {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl BackgroundColor for SolidBackgroundColor {
    fn color(&self, _ray: &Ray) -> Color {
        self.color
    }
}
