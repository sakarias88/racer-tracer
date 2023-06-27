use std::time::Duration;

use synchronoise::SignalEvent;

use crate::{
    background_color::BackgroundColor,
    camera::CameraSharedData,
    config::{Config, RenderConfig, RendererConfig},
    data_bus::DataWriter,
    error::TracerError,
    geometry::Hittable,
    image::Image,
    image_buffer::ImageBufferEvent,
    ray::Ray,
    vec3::{Color, Vec3},
};

use self::{cpu::CpuRenderer, cpu_scaled::CpuRendererScaled};

pub mod cpu;
pub mod cpu_scaled;
pub mod denoised;
pub mod image;

fn do_cancel(cancel_event: Option<&SignalEvent>) -> bool {
    match cancel_event {
        Some(event) => event.wait_timeout(Duration::from_secs(0)),
        None => false,
    }
}

#[derive(Default)]
pub struct RayImageData {
    rgb: Color,
    normal: Vec3,
    pos: Vec3,
    depth: f64,
    obj_id: usize,
}

fn ray_color(
    scene: &dyn Hittable,
    ray: &Ray,
    background: &dyn BackgroundColor,
    depth: usize,
    camera_pos: &Vec3,
) -> RayImageData {
    if depth == 0 {
        return RayImageData {
            rgb: Color::new(1.0, 1.0, 1.0),
            normal: Vec3::default(),
            pos: Vec3::default(),
            depth: f64::MAX,
            obj_id: 0,
        };
    }

    match scene.hit(ray, 0.001, std::f64::INFINITY) {
        Some(rec) => {
            let emitted = rec.material.color_emitted(rec.u, rec.v, &rec.point);
            let color = rec
                .material
                .scatter(ray, &rec)
                .map(|(scattered, attenuation)| {
                    emitted
                        + attenuation
                            * ray_color(scene, &scattered, background, depth - 1, camera_pos).rgb
                })
                .unwrap_or(emitted);
            RayImageData {
                rgb: color,
                normal: rec.normal,
                pos: rec.point,
                depth: (rec.point - camera_pos).length(),
                obj_id: rec.obj_id,
            }
        }
        None => {
            // TODO:
            // Faking data here for now.
            RayImageData {
                rgb: background.color(ray),
                normal: Vec3::default(),
                pos: Vec3::default(),
                depth: f64::MAX,
                obj_id: 0,
            }
        }
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
        writer: &DataWriter<ImageBufferEvent>,
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
