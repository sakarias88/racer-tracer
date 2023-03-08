use crate::image::Image;
use crate::ray::Ray;
use crate::util::degrees_to_radians;
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
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(
        look_from: Vec3,
        look_at: Vec3,
        up: Vec3,
        vfov: f64,
        image: &Image,
        focal_length: f64,
    ) -> Camera {
        let h = (degrees_to_radians(vfov) / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = image.aspect_ratio * viewport_height;

        let forward = (look_from - look_at).unit_vector();
        let right = up.cross(&forward).unit_vector();
        let up = forward.cross(&right);

        let horizontal = viewport_width * right;
        let vertical = viewport_height * up;

        Camera {
            viewport_height,
            viewport_width,
            focal_length,
            origin: look_from,
            horizontal,
            vertical,
            upper_left_corner: look_from + vertical / 2.0 - horizontal / 2.0 - forward,
            forward,
            right,
            up,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.upper_left_corner + u * self.horizontal - v * self.vertical - self.origin,
        )
    }

    // TODO: Add support for rotation

    // TODO: Use forward facing vector
    pub fn go_forward(&mut self, go: f64) {
        self.origin += self.forward * go;

        self.upper_left_corner =
            self.origin + self.vertical / 2.0 - self.horizontal / 2.0 - self.forward;
    }

    // TODO: Use right facing vector
    pub fn go_right(&mut self, go: f64) {
        self.origin += self.right * go;

        self.upper_left_corner =
            self.origin + self.vertical / 2.0 - self.horizontal / 2.0 - self.forward;
    }
}
