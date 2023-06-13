use std::sync::RwLock;

use rayon::prelude::*;

use crate::{
    camera::{Camera, CameraSharedData},
    error::TracerError,
    image::{Image, SubImage},
    renderer::{do_cancel, ray_color, Renderer},
    util::random_double,
    vec3::{Color, Vec3},
};

use super::{ImageData, RenderData};

pub struct CpuRenderer {
    rgb_buffer: RwLock<Vec<Color>>,
}

impl CpuRenderer {
    pub fn new(image: &Image) -> Self {
        Self {
            rgb_buffer: RwLock::new(vec![Vec3::default(); image.height * image.width]),
        }
    }

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
                        rd.background,
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
                self.rgb_buffer
                    .write()
                    .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
            })
            .map(|mut buf| {
                let offset = image.y * image.screen_width + image.x;
                for row in 0..image.height {
                    for col in 0..image.width {
                        buf[offset + row * image.screen_width + col] =
                            colors[row * image.width + col].scale_sqrt(rd.config.render.samples);
                    }
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

    fn image_data(&self) -> Result<ImageData, TracerError> {
        self.rgb_buffer
            .read()
            .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
            .map(|v| ImageData { rgb: v.clone() })
    }
}
