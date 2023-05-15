use rayon::prelude::*;

use crate::{
    camera::{Camera, CameraSharedData},
    error::TracerError,
    image::SubImage,
    renderer::{cpu::CpuRenderer, do_cancel, ray_color, Renderer},
    util::random_double,
    vec3::Vec3,
};

use super::RenderData;

fn get_highest_divdable(value: usize, mut div: usize) -> usize {
    // Feels like there could possibly be some other nicer trick to this.
    while (value % div) != 0 {
        div -= 1;
    }
    div
}

pub struct CpuRendererScaled {}

impl CpuRendererScaled {
    pub fn raytrace(
        &self,
        rd: &RenderData,
        camera_data: &CameraSharedData,
        image: &SubImage,
        scale: (usize, usize),
    ) -> Result<(), TracerError> {
        let (scale_width, scale_height) = scale;
        let scaled_width = image.width / scale_width;
        let scaled_height = image.height / scale_height;

        let mut colors: Vec<Vec3> = vec![Vec3::default(); scaled_height * scaled_width];
        for row in 0..scaled_height {
            for column in 0..scaled_width {
                let u: f64 = ((image.x + column * scale_width) as f64 + random_double())
                    / (image.screen_width - 1) as f64;
                for _ in 0..rd.config.preview.samples {
                    let v: f64 = ((image.y + row * scale_height) as f64 + random_double())
                        / (image.screen_height - 1) as f64;
                    colors[row * scaled_width + column].add(ray_color(
                        rd.scene,
                        &Camera::get_ray(camera_data, u, v),
                        rd.background,
                        rd.config.preview.max_depth,
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
                for scaled_row in 0..scaled_height {
                    for scaled_col in 0..scaled_width {
                        let color = colors[scaled_row * scaled_width + scaled_col]
                            .scale_sqrt(rd.config.preview.samples)
                            .as_color();
                        let row = scaled_row * scale_height;
                        let col = scaled_col * scale_width;
                        for scale_h in 0..scale_height {
                            for scale_w in 0..scale_width {
                                buf[offset
                                    + (row + scale_h) * image.screen_width
                                    + col
                                    + scale_w] = color;
                            }
                        }
                    }
                }
                if let Some(updated) = rd.buffer_updated {
                    updated.signal()
                }
            })
    }
}

impl Renderer for CpuRendererScaled {
    fn render(&self, rd: RenderData) -> Result<(), crate::error::TracerError> {
        let scale_width = get_highest_divdable(
            rd.image.width / rd.config.render.num_threads_width,
            rd.config.preview.scale,
        );
        let scale_height = get_highest_divdable(
            rd.image.height / rd.config.render.num_threads_height,
            rd.config.preview.scale,
        );

        CpuRenderer::prepare_threads(&rd).and_then(|images| {
            images
                .into_par_iter()
                .map(|image| {
                    self.raytrace(&rd, rd.camera_data, &image, (scale_width, scale_height))
                })
                .collect::<Result<(), TracerError>>()
        })
    }
}
