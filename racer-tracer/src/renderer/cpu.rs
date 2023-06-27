use rayon::prelude::*;

use crate::{
    camera::{Camera, CameraSharedData},
    config::RenderConfig,
    data_bus::DataWriter,
    error::TracerError,
    image::SubImage,
    image_buffer::ImageBufferEvent,
    renderer::{do_cancel, ray_color, Renderer},
    util::random_double,
    vec3::{Color, Vec3},
};

use super::RenderData;

pub struct CpuRenderer {
    config: RenderConfig,
}

impl CpuRenderer {
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    pub fn raytrace(
        &self,
        rd: &RenderData,
        camera_data: &CameraSharedData,
        image: SubImage<ImageBufferEvent>,
    ) -> Result<(), TracerError> {
        let mut buffer = vec![Vec3::default(); image.height * image.width];
        for row in 0..image.height {
            for column in 0..image.width {
                let u: f64 =
                    ((image.x + column) as f64 + random_double()) / (image.screen_width - 1) as f64;
                let mut color = Color::default();
                for _ in 0..rd.config.render.samples {
                    let v: f64 = ((image.y + row) as f64 + random_double())
                        / (image.screen_height - 1) as f64;
                    color.add(
                        ray_color(
                            rd.scene,
                            &Camera::get_ray(camera_data, u, v),
                            rd.background,
                            rd.config.render.max_depth,
                            &camera_data.origin,
                        )
                        .rgb,
                    );
                }
                buffer[row * image.width + column] = color.scale_sqrt(self.config.samples);
            }

            if do_cancel(rd.cancel_event) {
                return Ok(());
            }
        }

        if do_cancel(rd.cancel_event) {
            return Ok(());
        }

        image.writer.write(ImageBufferEvent::BufferUpdate {
            rgb: buffer,
            r: image.y,
            c: image.x,
            width: image.width,
            height: image.height,
        })
    }

    pub fn prepare_threads<T: Clone>(
        rd: &RenderData,
        conf: &RenderConfig,
        writer: &DataWriter<T>,
    ) -> Result<Vec<SubImage<T>>, TracerError> {
        let width_step = rd.image.width / conf.num_threads_width;
        let height_step = rd.image.height / conf.num_threads_height;

        (!do_cancel(rd.cancel_event))
            .then_some(|| ())
            .ok_or(TracerError::CancelEvent)
            .map(|_| {
                (0..conf.num_threads_width)
                    .flat_map(|ws| {
                        (0..conf.num_threads_height)
                            .map(|hs| SubImage {
                                writer: writer.clone(),
                                x: width_step * ws,
                                y: height_step * hs,
                                screen_width: rd.image.width,
                                screen_height: rd.image.height,

                                // Neccesary in case the threads width is not
                                // evenly divisible by the image width.
                                width: if ws == conf.num_threads_width - 1 {
                                    rd.image.width - width_step * ws
                                } else {
                                    width_step
                                },

                                // Neccesary in case the threads height is not
                                // evenly divisible by the image height.
                                height: if hs == conf.num_threads_height - 1 {
                                    rd.image.height - height_step * hs
                                } else {
                                    height_step
                                },
                            })
                            .collect::<Vec<SubImage<T>>>()
                    })
                    .collect::<Vec<SubImage<T>>>()
            })
    }
}

impl Renderer for CpuRenderer {
    fn render(
        &self,
        rd: RenderData,
        writer: &DataWriter<ImageBufferEvent>,
    ) -> Result<(), TracerError> {
        CpuRenderer::prepare_threads(&rd, &self.config, writer).and_then(|images| {
            images
                .into_par_iter()
                .map(|image| self.raytrace(&rd, rd.camera_data, image))
                .collect::<Result<(), TracerError>>()
        })
    }
}
