use crate::image::Image;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Clone)]
pub struct Camera {
    pub viewport_height: f64,
    pub viewport_width: f64,
    pub focal_length: f64,
    pub origin: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub upper_left_corner: Vec3,
}

impl Camera {
    pub fn new(image: &Image, viewport_height: f64, focal_length: f64) -> Camera {
        let viewport_width = image.aspect_ratio * viewport_height;
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        Camera {
            viewport_height,
            viewport_width,
            focal_length,
            origin,
            horizontal,
            vertical,
            upper_left_corner: origin + vertical / 2.0
                - horizontal / 2.0
                - Vec3::new(0.0, 0.0, focal_length),
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.upper_left_corner + u * self.horizontal - v * self.vertical - self.origin,
        )
    }
}
