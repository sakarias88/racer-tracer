use crate::config::CameraConfig;
use crate::data_bus::{DataBus, DataReader, DataWriter};
use crate::error::TracerError;
use crate::image::Image;
use crate::ray::Ray;
use crate::util::{degrees_to_radians, random_double_range, random_in_unit_disk};
use crate::vec3::Vec3;

#[derive(Clone)]
pub enum CameraEvent {
    Pos {
        origin: Vec3,
        upper_left_corner: Vec3,
    },
    LookAt {
        forward: Vec3,
        right: Vec3,
        up: Vec3,
        horizontal: Vec3,
        vertical: Vec3,
        upper_left_corner: Vec3,
    },
    Vfov {
        vfov: f64,
        viewport_height: f64,
        viewport_width: f64,
        horizontal: Vec3,
        vertical: Vec3,
        upper_left_corner: Vec3,
    },
    Aperture {
        lens_radius: f64,
    },
    FocusDistance {
        focus_distance: f64,
        horizontal: Vec3,
        vertical: Vec3,
        upper_left_corner: Vec3,
    },
    Rotate {
        forward: Vec3,
        right: Vec3,
        up: Vec3,
        vertical: Vec3,
        horizontal: Vec3,
        upper_left_corner: Vec3,
    },
}

pub struct SharedCamera {
    reader: DataReader<CameraEvent>,
    data: CameraSharedData,
    changed: bool,
}

#[derive(Clone)]
pub struct CameraSharedData {
    pub origin: Vec3,
    upper_left_corner: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    vfov: f64,
    viewport_width: f64,
    viewport_height: f64,
    lens_radius: f64,
    focus_distance: f64,
    time_a: f64,
    time_b: f64,
}

impl SharedCamera {
    pub fn new(data: CameraSharedData, reader: DataReader<CameraEvent>) -> Self {
        Self {
            reader,
            data,
            changed: true,
        }
    }

    #[allow(dead_code)]
    pub fn pos(&self) -> &Vec3 {
        &self.data.origin
    }

    pub fn data(&self) -> &CameraSharedData {
        &self.data
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.changed = false;
        self.reader.get_messages().map(|messages| {
            messages.into_iter().for_each(|action| {
                self.changed = true;
                match action {
                    CameraEvent::Pos {
                        origin,
                        upper_left_corner,
                    } => {
                        self.data.origin = origin;
                        self.data.upper_left_corner = upper_left_corner;
                    }
                    CameraEvent::LookAt {
                        forward,
                        right,
                        up,
                        horizontal,
                        vertical,
                        upper_left_corner,
                    } => {
                        self.data.forward = forward;
                        self.data.right = right;
                        self.data.up = up;
                        self.data.horizontal = horizontal;
                        self.data.vertical = vertical;
                        self.data.upper_left_corner = upper_left_corner;
                    }
                    CameraEvent::Vfov {
                        vfov,
                        viewport_height,
                        viewport_width,
                        horizontal,
                        vertical,
                        upper_left_corner,
                    } => {
                        self.data.vfov = vfov;
                        self.data.viewport_height = viewport_height;
                        self.data.viewport_width = viewport_width;
                        self.data.horizontal = horizontal;
                        self.data.vertical = vertical;
                        self.data.upper_left_corner = upper_left_corner;
                    }
                    CameraEvent::Aperture { lens_radius } => {
                        self.data.lens_radius = lens_radius;
                    }
                    CameraEvent::FocusDistance {
                        focus_distance,
                        horizontal,
                        vertical,
                        upper_left_corner,
                    } => {
                        self.data.focus_distance = focus_distance;
                        self.data.horizontal = horizontal;
                        self.data.vertical = vertical;
                        self.data.upper_left_corner = upper_left_corner;
                    }
                    CameraEvent::Rotate {
                        forward,
                        right,
                        up,
                        vertical,
                        horizontal,
                        upper_left_corner,
                    } => {
                        self.data.forward = forward;
                        self.data.right = right;
                        self.data.up = up;
                        self.data.vertical = vertical;
                        self.data.horizontal = horizontal;
                        self.data.upper_left_corner = upper_left_corner;
                    }
                };
            });
        })
    }
}

pub struct CameraInitData {
    pub look_from: Vec3,
    pub look_at: Vec3,
    pub scene_up: Vec3,
    pub vfov: f64,
    pub aperture: f64,
    pub focus_distance: f64,
    pub aspect_ratio: f64,
    pub time_a: f64,
    pub time_b: f64,
}

pub struct Camera {
    bus: DataBus<CameraEvent>,
    writer: DataWriter<CameraEvent>,
    data: CameraSharedData,
    scene_up: Vec3,
    aspect_ratio: f64,
    aperture: f64,
}

impl Camera {
    pub fn new(init: CameraInitData, image: &Image) -> Self {
        let h = (degrees_to_radians(init.vfov) / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = image.aspect_ratio * viewport_height;

        let forward = (init.look_from - init.look_at).unit_vector();
        let right = init.scene_up.cross(&forward).unit_vector();
        let up = forward.cross(&right);

        let horizontal = init.focus_distance * viewport_width * right;
        let vertical = init.focus_distance * viewport_height * up;

        let bus = DataBus::<CameraEvent>::new("camera");
        Self {
            writer: bus.get_writer(),
            bus,
            data: CameraSharedData {
                viewport_height,
                viewport_width,
                origin: init.look_from,
                horizontal,
                vertical,
                upper_left_corner: init.look_from + vertical / 2.0
                    - horizontal / 2.0
                    - init.focus_distance * forward,
                forward,
                right,
                up,
                vfov: init.vfov,
                lens_radius: init.aperture * 0.5,
                focus_distance: init.focus_distance,
                time_a: init.time_a,
                time_b: init.time_b,
            },
            aspect_ratio: init.aspect_ratio,
            scene_up: init.scene_up,
            aperture: init.aperture,
        }
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.bus.update()
    }

    pub fn get_shared_camera(&mut self) -> SharedCamera {
        SharedCamera::new(self.data.clone(), self.bus.get_reader())
    }

    pub fn forward(&self) -> Vec3 {
        self.data.forward
    }

    pub fn up(&self) -> Vec3 {
        self.data.up
    }

    pub fn right(&self) -> Vec3 {
        self.data.right
    }

    pub fn set_pos(&mut self, pos: Vec3) -> Result<(), TracerError> {
        self.data.origin = pos;
        self.update_corner();
        self.writer.write(CameraEvent::Pos {
            origin: self.data.origin,
            upper_left_corner: self.data.upper_left_corner,
        })
    }

    #[allow(dead_code)]
    pub fn set_look_at(&mut self, look_at: Vec3) -> Result<(), TracerError> {
        self.data.forward = (self.data.origin - look_at).unit_vector();
        self.update_directions();
        self.writer.write(CameraEvent::LookAt {
            forward: self.data.forward,
            right: self.data.right,
            up: self.data.up,
            horizontal: self.data.horizontal,
            vertical: self.data.vertical,
            upper_left_corner: self.data.vertical,
        })
    }

    pub fn set_fov(&mut self, vfov: f64) -> Result<(), TracerError> {
        self.data.vfov = vfov;
        let h = (degrees_to_radians(self.data.vfov) / 2.0).tan();
        self.data.viewport_height = 2.0 * h;
        self.data.viewport_width = self.aspect_ratio * self.data.viewport_height;
        self.update_viewport();
        self.writer.write(CameraEvent::Vfov {
            vfov: self.data.vfov,
            viewport_height: self.data.viewport_height,
            viewport_width: self.data.viewport_width,
            horizontal: self.data.horizontal,
            vertical: self.data.vertical,
            upper_left_corner: self.data.upper_left_corner,
        })
    }

    pub fn get_vfov(&self) -> f64 {
        self.data.vfov
    }

    pub fn set_aperture(&mut self, aperture: f64) -> Result<(), TracerError> {
        self.aperture = aperture;
        self.data.lens_radius = self.aperture * 0.5;
        self.writer.write(CameraEvent::Aperture {
            lens_radius: self.data.lens_radius,
        })
    }

    pub fn get_aperture(&self) -> f64 {
        self.aperture
    }

    pub fn set_focus_distance(&mut self, focus_distance: f64) -> Result<(), TracerError> {
        self.data.focus_distance = focus_distance;
        self.update_viewport();
        self.writer.write(CameraEvent::FocusDistance {
            focus_distance: self.data.focus_distance,
            horizontal: self.data.horizontal,
            vertical: self.data.vertical,
            upper_left_corner: self.data.upper_left_corner,
        })
    }

    pub fn get_focus_distance(&self) -> f64 {
        self.data.focus_distance
    }

    pub fn get_ray(camera_data: &CameraSharedData, u: f64, v: f64) -> Ray {
        let ray_direction = camera_data.lens_radius * random_in_unit_disk();
        let offset = camera_data.right * ray_direction.x() + camera_data.up * ray_direction.y();
        Ray::new(
            camera_data.origin + offset,
            camera_data.upper_left_corner + u * camera_data.horizontal
                - v * camera_data.vertical
                - camera_data.origin
                - offset,
            random_double_range(camera_data.time_a, camera_data.time_b),
        )
    }

    pub fn go_forward(&mut self, go: f64) -> Result<(), TracerError> {
        self.set_pos(self.data.origin + self.data.forward * go)
    }

    pub fn go_right(&mut self, go: f64) -> Result<(), TracerError> {
        self.set_pos(self.data.origin + self.data.right * go)
    }

    pub fn rotate(&mut self, right_move: f64, up_move: f64) -> Result<(), TracerError> {
        self.data.forward.rotate(up_move, &self.data.right);
        self.data.forward.rotate(right_move, &self.scene_up);
        self.update_directions();
        self.update_corner();
        self.writer.write(CameraEvent::Rotate {
            forward: self.data.forward,
            right: self.data.right,
            up: self.data.up,
            vertical: self.data.vertical,
            horizontal: self.data.horizontal,
            upper_left_corner: self.data.upper_left_corner,
        })
    }

    #[allow(dead_code)]
    pub fn rotate_right(&mut self, right_move: f64) -> Result<(), TracerError> {
        self.rotate(right_move, 0.0)
    }

    #[allow(dead_code)]
    pub fn rotate_up(&mut self, up_move: f64) -> Result<(), TracerError> {
        self.rotate(0.0, up_move)
    }

    fn update_directions(&mut self) {
        self.data.forward.unit_vector();
        self.data.right = self.scene_up.cross(&self.data.forward).unit_vector();
        self.data.up = self.data.forward.cross(&self.data.right);
        self.update_viewport();
    }

    fn update_viewport(&mut self) {
        self.data.horizontal =
            self.data.focus_distance * self.data.viewport_width * self.data.right;
        self.data.vertical = self.data.focus_distance * self.data.viewport_height * self.data.up;
        self.update_corner()
    }

    fn update_corner(&mut self) {
        self.data.upper_left_corner = self.data.origin + self.data.vertical / 2.0
            - self.data.horizontal / 2.0
            - self.data.focus_distance * self.data.forward;
    }
}

pub struct CameraData {
    pub vfov: f64,
    pub aperture: f64,
    pub focus_distance: f64,
    pub pos: Vec3,
    pub look_at: Vec3,
    pub speed: f64,
    pub sensitivity: f64,
}

impl CameraData {
    pub fn merge(data1: CameraConfig, data2: CameraConfig) -> Self {
        Self {
            vfov: data1
                .vfov
                .or(data2.vfov)
                .unwrap_or_else(CameraData::default_vfov),
            aperture: data1
                .aperture
                .or(data2.aperture)
                .unwrap_or_else(CameraData::default_aperture),
            focus_distance: data1
                .focus_distance
                .or(data2.focus_distance)
                .unwrap_or_else(CameraData::default_focus_distance),
            pos: data1
                .pos
                .or(data2.pos)
                .unwrap_or_else(CameraData::default_pos),
            look_at: data1
                .look_at
                .or(data2.look_at)
                .unwrap_or_else(CameraData::default_look_at),
            speed: data1
                .speed
                .or(data2.speed)
                .unwrap_or_else(CameraData::default_speed),
            sensitivity: data1
                .sensitivity
                .or(data2.sensitivity)
                .unwrap_or_else(CameraData::default_sensitivity),
        }
    }

    pub fn default_vfov() -> f64 {
        20.0
    }

    pub fn default_aperture() -> f64 {
        0.0
    }

    pub fn default_focus_distance() -> f64 {
        1000.0
    }

    pub fn default_pos() -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    pub fn default_look_at() -> Vec3 {
        Vec3::new(0.0, 0.0, -1.0)
    }

    pub fn default_speed() -> f64 {
        0.0002
    }

    pub fn default_sensitivity() -> f64 {
        0.001
    }
}
