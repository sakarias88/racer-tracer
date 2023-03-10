use crate::image::Image;
use crate::ray::Ray;
use crate::util::{degrees_to_radians, random_in_unit_disk};
use crate::vec3::Vec3;

#[derive(Clone)]
pub struct Camera {
    pub viewport_height: f64,
    pub viewport_width: f64,
    pub origin: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub upper_left_corner: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub lens_radius: f64,
    pub focus_distance: f64,
}

impl Camera {
    pub fn new(
        look_from: Vec3,
        look_at: Vec3,
        up: Vec3,
        vfov: f64,
        image: &Image,
        aperture: f64,
        focus_distance: f64,
    ) -> Camera {
        let h = (degrees_to_radians(vfov) / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = image.aspect_ratio * viewport_height;

        let forward = (look_from - look_at).unit_vector();
        let right = up.cross(&forward).unit_vector();
        let up = forward.cross(&right);

        let horizontal = focus_distance * viewport_width * right;
        let vertical = focus_distance * viewport_height * up;
        Camera {
            viewport_height,
            viewport_width,
            origin: look_from,
            horizontal,
            vertical,
            upper_left_corner: look_from + vertical / 2.0
                - horizontal / 2.0
                - focus_distance * forward,
            forward,
            right,
            up,
            lens_radius: aperture * 0.5,
            focus_distance,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let ray_direction = self.lens_radius * random_in_unit_disk();
        let offset = self.right * ray_direction.x() + self.up * ray_direction.y();
        Ray::new(
            self.origin + offset,
            self.upper_left_corner + u * self.horizontal - v * self.vertical - self.origin - offset,
        )
    }

    pub fn go_forward(&mut self, go: f64) {
        self.origin += self.forward * go;

        self.upper_left_corner = self.origin + self.vertical / 2.0
            - self.horizontal / 2.0
            - self.focus_distance * self.forward;
    }

    pub fn go_right(&mut self, go: f64) {
        self.origin += self.right * go;

        self.upper_left_corner = self.origin + self.vertical / 2.0
            - self.horizontal / 2.0
            - self.focus_distance * self.forward;
    }
}
