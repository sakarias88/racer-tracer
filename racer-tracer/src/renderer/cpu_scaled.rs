use rayon::prelude::*;

use crate::{
    camera::{Camera, CameraSharedData},
    config::RenderConfig,
    error::TracerError,
    gbuffer::ImageBufferWriter,
    image::{Image, SubImage},
    renderer::{cpu::CpuRenderer, do_cancel, ray_color, Renderer},
    util::random_double,
    vec3::{Color, Vec3},
};

use super::RenderData;

fn get_highest_divdable(value: usize, mut div: usize) -> usize {
    // Feels like there could possibly be some other nicer trick to this.
    while (value % div) != 0 {
        div -= 1;
    }
    div
}

pub struct CpuRendererScaled {
    config: RenderConfig,
    scale_width: usize,
    scale_height: usize,
}

impl CpuRendererScaled {
    pub fn new(config: RenderConfig, image: &Image) -> Self {
        let scale_width =
            get_highest_divdable(image.width / config.num_threads_width, config.scale);
        let scale_height =
            get_highest_divdable(image.height / config.num_threads_height, config.scale);

        Self {
            config,
            scale_width,
            scale_height,
        }
    }

    pub fn raytrace(
        &self,
        rd: &RenderData,
        camera_data: &CameraSharedData,
        mut image: SubImage,
    ) -> Result<(), TracerError> {
        let scaled_width = image.width / self.scale_width;
        let scaled_height = image.height / self.scale_height;
        let mut buffer = vec![Vec3::default(); image.height * image.width];

        for row in 0..scaled_height {
            for column in 0..scaled_width {
                let u: f64 = ((image.x + column * self.scale_width) as f64 + random_double())
                    / (image.screen_width - 1) as f64;
                let mut color = Color::default();
                for _ in 0..self.config.samples {
                    let v: f64 = ((image.y + row * self.scale_height) as f64 + random_double())
                        / (image.screen_height - 1) as f64;
                    color.add(ray_color(
                        rd.scene,
                        &Camera::get_ray(camera_data, u, v),
                        rd.background,
                        self.config.max_depth,
                    ));
                }

                // Scale up color
                color = color.scale_sqrt(self.config.samples);
                let upscaled_row = row * self.scale_height;
                let upscaled_col = column * self.scale_width;
                for scale_h in 0..self.scale_height {
                    for scale_w in 0..self.scale_width {
                        buffer[(scale_h + upscaled_row) * image.width + scale_w + upscaled_col] =
                            color;
                    }
                }
            }

            if do_cancel(rd.cancel_event) {
                return Ok(());
            }
        }

        image
            .writer
            .write(buffer, image.y, image.x, image.width, image.height)
    }
}

impl Renderer for CpuRendererScaled {
    fn render(
        &self,
        rd: RenderData,
        writer: &ImageBufferWriter,
    ) -> Result<(), crate::error::TracerError> {
        CpuRenderer::prepare_threads(&rd, &self.config, writer).and_then(|images| {
            images
                .into_par_iter()
                .map(|image| self.raytrace(&rd, rd.camera_data, image))
                .collect::<Result<(), TracerError>>()
        })
    }
}
