use crate::config::CameraConfig;
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
    pub scene_up: Vec3,
    pub lens_radius: f64,
    pub focus_distance: f64,
    pub aspect_ratio: f64,
}

impl Camera {
    pub fn new(
        look_from: Vec3,
        look_at: Vec3,
        scene_up: Vec3,
        vfov: f64,
        image: &Image,
        aperture: f64,
        focus_distance: f64,
    ) -> Camera {
        let h = (degrees_to_radians(vfov) / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = image.aspect_ratio * viewport_height;

        let forward = (look_from - look_at).unit_vector();
        let right = scene_up.cross(&forward).unit_vector();
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
            scene_up,
            lens_radius: aperture * 0.5,
            focus_distance,
            aspect_ratio: image.aspect_ratio,
        }
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.origin = pos;
        self.update_corner();
    }

    pub fn set_look_at(&mut self, look_at: Vec3) {
        self.forward = (self.origin - look_at).unit_vector();
        self.update_directions();
    }

    pub fn set_fov(&mut self, vfov: f64) {
        let h = (degrees_to_radians(vfov) / 2.0).tan();
        self.viewport_height = 2.0 * h;
        self.viewport_width = self.aspect_ratio * self.viewport_height;
        self.update_viewport();
    }

    pub fn set_aperture(&mut self, aperture: f64) {
        self.lens_radius = aperture * 0.5;
    }

    pub fn set_focus_distance(&mut self, focus_distance: f64) {
        self.focus_distance = focus_distance;
        self.update_viewport();
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
        self.update_corner()
    }

    pub fn go_right(&mut self, go: f64) {
        self.origin += self.right * go;
        self.update_corner()
    }

    pub fn rotate(&mut self, right_move: f64, up_move: f64) {
        self.forward.rotate(up_move, &self.right);
        self.forward.rotate(right_move, &self.scene_up);
        self.update_directions();
        self.update_corner();
    }

    pub fn rotate_up(&mut self, degrees: f64) {
        self.forward.rotate(degrees, &self.right);
        self.update_directions();
    }

    pub fn rotate_right(&mut self, degrees: f64) {
        self.forward.rotate(degrees, &self.scene_up);
        self.update_directions();
    }

    fn update_directions(&mut self) {
        self.forward.unit_vector();
        self.right = self.scene_up.cross(&self.forward).unit_vector();
        self.up = self.forward.cross(&self.right);
        self.update_viewport();
    }

    fn update_viewport(&mut self) {
        self.horizontal = self.focus_distance * self.viewport_width * self.right;
        self.vertical = self.focus_distance * self.viewport_height * self.up;
        self.update_corner()
    }

    fn update_corner(&mut self) {
        self.upper_left_corner = self.origin + self.vertical / 2.0
            - self.horizontal / 2.0
            - self.focus_distance * self.forward;
    }
}

impl From<(&Image, &CameraConfig)> for Camera {
    fn from((image, c): (&Image, &CameraConfig)) -> Self {
        Self::new(
            c.pos,
            c.look_at,
            Vec3::new(0.0, 1.0, 0.0),
            c.vfov,
            image,
            c.aperture,
            c.focus_distance,
        )
    }
}
