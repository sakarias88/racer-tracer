// Just for comparing images with different tone maps.

use std::path::Path;

use image::ImageBuffer;

use crate::{error::TracerError, vec3::Color};

use super::{RenderData, Renderer};

pub struct ImageDisplayer {
    img: ImageBuffer<img::Rgba<u8>, std::vec::Vec<u8>>,
}

impl ImageDisplayer {
    #[allow(dead_code)]
    pub fn try_new(path: &Path) -> Result<Self, TracerError> {
        image::open(path)
            .map_err(|e| {
                TracerError::FailedToOpenImage(path.to_string_lossy().into_owned(), e.to_string())
            })
            .map(|v| Self {
                img: v.into_rgba8(),
            })
    }
}

impl Renderer for ImageDisplayer {
    fn render(&self, rd: RenderData) -> Result<(), crate::error::TracerError> {
        let mut buffer = rd
            .buffer
            .write()
            .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))?;

        for row in 0..rd.image.height {
            for col in 0..rd.image.width {
                let color = if row >= self.img.height() as usize || col >= self.img.width() as usize
                {
                    Color::default()
                } else {
                    let pixel = self.img.get_pixel(col as u32, row as u32);
                    let color_scale = 1.0 / 255.0;
                    rd.tone_mapping.tone_map(Color::new(
                        f64::from(pixel.0[0]) * color_scale,
                        f64::from(pixel.0[1]) * color_scale,
                        f64::from(pixel.0[2]) * color_scale,
                    ))
                };
                buffer[row * rd.image.width + col] = color.as_color();
            }
        }
        Ok(())
    }
}
