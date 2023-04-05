use std::{sync::RwLock, time::Duration};

use synchronoise::SignalEvent;

use crate::{
    camera::Camera, config::Config, error::TracerError, geometry::Hittable, image::Image, ray::Ray,
    vec3::Color, vec3::Vec3,
};

pub mod cpu;
pub mod cpu_scaled;

fn do_cancel(cancel_event: Option<&SignalEvent>) -> bool {
    match cancel_event {
        Some(event) => event.wait_timeout(Duration::from_secs(0)),
        None => false,
    }
}

fn ray_color(scene: &dyn Hittable, ray: &Ray, depth: usize) -> Vec3 {
    if depth == 0 {
        return Vec3::default();
    }

    if let Some(rec) = scene.hit(ray, 0.001, std::f64::INFINITY) {
        if let Some((scattered, attenuation)) = rec.material.scatter(ray, &rec) {
            return attenuation * ray_color(scene, &scattered, depth - 1);
        }
        return Color::default();
    }

    // Sky
    let first_color = Vec3::new(1.0, 1.0, 1.0);
    let second_color = Vec3::new(0.5, 0.7, 1.0);
    let unit_direction = ray.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * first_color + t * second_color
}

pub struct RenderData<'a> {
    pub buffer: &'a RwLock<Vec<u32>>,
    pub camera: &'a RwLock<Camera>,
    pub image: &'a Image,
    pub scene: &'a dyn Hittable,
    pub config: &'a Config,
    pub cancel_event: Option<&'a SignalEvent>,
}

pub trait Renderer: Send + Sync {
    fn render(&self, render_data: RenderData) -> Result<(), TracerError>;
}
