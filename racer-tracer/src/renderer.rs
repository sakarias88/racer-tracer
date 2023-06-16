use std::time::Duration;

use synchronoise::SignalEvent;

use crate::{
    background_color::BackgroundColor,
    camera::CameraSharedData,
    config::{Config, RenderConfig, RendererConfig},
    error::TracerError,
    geometry::Hittable,
    image::Image,
    image_buffer::ImageBufferWriter,
    ray::Ray,
    vec3::{Color, Vec3},
};

use self::{cpu::CpuRenderer, cpu_scaled::CpuRendererScaled};

pub mod cpu;
pub mod cpu_scaled;
pub mod image;

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
) -> Color {
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
    pub camera_data: &'a CameraSharedData,
    pub image: &'a Image,
    pub scene: &'a dyn Hittable,
    pub background: &'a dyn BackgroundColor,
    pub config: &'a Config,
    pub cancel_event: Option<&'a SignalEvent>,
}

pub trait Renderer: Send + Sync {
    fn render(
        &self,
        render_data: RenderData,
        gbuffer_writer: &ImageBufferWriter,
    ) -> Result<(), TracerError>;
}

impl From<(&RendererConfig, &RenderConfig, &Image)> for Box<dyn Renderer> {
    fn from(r: (&RendererConfig, &RenderConfig, &Image)) -> Self {
        match r.0 {
            RendererConfig::Cpu => Box::new(CpuRenderer::new(r.1.clone())),
            RendererConfig::CpuPreview => Box::new(CpuRendererScaled::new(r.1.clone(), r.2)),
        }
    }
}
