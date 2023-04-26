use rayon::prelude::*;

use crate::{
    camera::{Camera, CameraSharedData},
    error::TracerError,
    image::SubImage,
    renderer::{do_cancel, ray_color, Renderer},
    util::random_double,
    vec3::Vec3,
};

use super::RenderData;

pub struct CpuRenderer {}

impl CpuRenderer {
    pub fn raytrace(
        &self,
        rd: &RenderData,
        camera_data: &CameraSharedData,
        image: &SubImage,
    ) -> Result<(), TracerError> {
        let mut colors: Vec<Vec3> = vec![Vec3::default(); image.height * image.width];
        for row in 0..image.height {
            for column in 0..image.width {
                let u: f64 =
                    ((image.x + column) as f64 + random_double()) / (image.screen_width - 1) as f64;
                for _ in 0..rd.config.render.samples {
                    let v: f64 = ((image.y + row) as f64 + random_double())
                        / (image.screen_height - 1) as f64;
                    colors[row * image.width + column].add(ray_color(
                        rd.scene,
                        &Camera::get_ray(camera_data, u, v),
                        rd.config.render.max_depth,
                    ));
                }
            }

            if do_cancel(rd.cancel_event) {
                return Ok(());
            }
        }

        (!do_cancel(rd.cancel_event))
            .then_some(|| ())
            .ok_or(TracerError::CancelEvent)
            .and_then(|_| {
                rd.buffer
                    .write()
                    .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
            })
            .map(|mut buf| {
                let offset = image.y * image.screen_width + image.x;
                for row in 0..image.height {
                    for col in 0..image.width {
                        let color = colors[row * image.width + col]
                            .scale_sqrt(rd.config.render.samples)
                            .as_color();
                        buf[offset + row * image.screen_width + col] = color;
                    }
                }
                if let Some(updated) = rd.buffer_updated {
                    updated.signal()
                }
            })
    }

    pub fn prepare_threads(rd: &RenderData) -> Result<Vec<SubImage>, TracerError> {
        let width_step = rd.image.width / rd.config.render.num_threads_width;
        let height_step = rd.image.height / rd.config.render.num_threads_height;

        (!do_cancel(rd.cancel_event))
            .then_some(|| ())
            .ok_or(TracerError::CancelEvent)
            .map(|_| {
                (0..rd.config.render.num_threads_width)
                    .flat_map(|ws| {
                        (0..rd.config.render.num_threads_height)
                            .map(|hs| SubImage {
                                x: width_step * ws,
                                y: height_step * hs,
                                screen_width: rd.image.width,
                                screen_height: rd.image.height,

                                // Neccesary in case the threads width is not
                                // evenly divisible by the image width.
                                width: if ws == rd.config.render.num_threads_width - 1 {
                                    rd.image.width - width_step * ws
                                } else {
                                    width_step
                                },

                                // Neccesary in case the threads height is not
                                // evenly divisible by the image height.
                                height: if hs == rd.config.render.num_threads_height - 1 {
                                    rd.image.height - height_step * hs
                                } else {
                                    height_step
                                },
                            })
                            .collect::<Vec<SubImage>>()
                    })
                    .collect::<Vec<SubImage>>()
            })
    }
}

impl Renderer for CpuRenderer {
    fn render(&self, rd: RenderData) -> Result<(), TracerError> {
        CpuRenderer::prepare_threads(&rd).and_then(|images| {
            images
                .into_par_iter()
                .map(|image| self.raytrace(&rd, rd.camera_data, &image))
                .collect::<Result<(), TracerError>>()
        })
    }
}
