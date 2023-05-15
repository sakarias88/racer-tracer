use std::{sync::RwLock, time::Duration};

use synchronoise::SignalEvent;

use crate::{
    background_color::BackgroundColor,
    camera::CameraSharedData,
    config::{Config, Renderer as ConfigRenderer},
    error::TracerError,
    geometry::Hittable,
    image::Image,
    ray::Ray,
    vec3::Vec3,
};

use self::{cpu::CpuRenderer, cpu_scaled::CpuRendererScaled};

pub mod cpu;
pub mod cpu_scaled;

fn do_cancel(cancel_event: Option<&SignalEvent>) -> bool {
    match cancel_event {
        Some(event) => event.wait_timeout(Duration::from_secs(0)),
        None => false,
    }
}

fn ray_color(
    scene: &dyn Hittable,
    ray: &Ray,
    background: &dyn BackgroundColor,
    depth: usize,
) -> Vec3 {
    if depth == 0 {
        return Vec3::default();
    }

    match scene.hit(ray, 0.001, std::f64::INFINITY) {
        Some(rec) => {
            let emitted = rec.material.color_emitted(rec.u, rec.v, &rec.point);
            match rec.material.scatter(ray, &rec) {
                Some((scattered, attenuation)) => {
                    emitted + attenuation * ray_color(scene, &scattered, background, depth - 1)
                }
                None => emitted,
            }
        }
        None => background.color(ray),
    }
}

pub struct RenderData<'a> {
    pub buffer: &'a RwLock<Vec<u32>>,
    pub camera_data: &'a CameraSharedData,
    pub image: &'a Image,
    pub scene: &'a dyn Hittable,
    pub background: &'a dyn BackgroundColor,
    pub config: &'a Config,
    pub cancel_event: Option<&'a SignalEvent>,
    pub buffer_updated: Option<&'a SignalEvent>,
}

pub trait Renderer: Send + Sync {
    fn render(&self, render_data: RenderData) -> Result<(), TracerError>;
}

impl From<&ConfigRenderer> for &dyn Renderer {
    fn from(r: &ConfigRenderer) -> Self {
        match r {
            ConfigRenderer::Cpu => &CpuRenderer {} as &dyn Renderer,
            ConfigRenderer::CpuPreview => &CpuRendererScaled {} as &dyn Renderer,
        }
    }
}
