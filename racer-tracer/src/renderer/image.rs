// Just for comparing images with different tone maps.

use std::path::Path;

use crate::{error::TracerError, gbuffer::ImageBufferWriter, image::Image, vec3::Color};

use super::{RenderData, Renderer};

pub struct ImageDisplayer {
    buffer: Vec<Color>,
}

impl ImageDisplayer {
    #[allow(dead_code)]
    pub fn try_new(path: &Path, image: &Image) -> Result<Self, TracerError> {
        image::open(path)
            .map_err(|e| {
                TracerError::FailedToOpenImage(path.to_string_lossy().into_owned(), e.to_string())
            })
            .map(|v| v.into_rgba8())
            .map(|image_buffer| {
                let mut rgb_buffer = Vec::with_capacity(image.width * image.height);
                for row in 0..image.height {
                    for col in 0..image.width {
                        if row >= image_buffer.height() as usize
                            || col >= image_buffer.width() as usize
                        {
                            rgb_buffer[row * image.width + col] = Color::default();
                        } else {
                            let pixel = image_buffer.get_pixel(col as u32, row as u32);
                            let color_scale = 1.0 / 255.0;
                            rgb_buffer[row * image.width + col] = Color::new(
                                f64::from(pixel.0[0]) * color_scale,
                                f64::from(pixel.0[1]) * color_scale,
                                f64::from(pixel.0[2]) * color_scale,
                            );
                        };
                    }
                }
                Self { buffer: rgb_buffer }
            })
    }
}

impl Renderer for ImageDisplayer {
    fn render(&self, rd: RenderData, writer: &ImageBufferWriter) -> Result<(), TracerError> {
        let mut writer = writer.clone();
        writer.write(self.buffer.clone(), 0, 0, rd.image.width, rd.image.height)
    }
}
