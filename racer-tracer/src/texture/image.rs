use std::path::Path;

use image::ImageBuffer;

use crate::{
    error::TracerError,
    vec3::{Color, Vec3},
};

use super::Texture;

pub struct TextureImage {
    img: ImageBuffer<img::Rgba<u8>, std::vec::Vec<u8>>,
}

impl TextureImage {
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

impl Texture for TextureImage {
    fn value(&self, u: f64, v: f64, _point: &Vec3) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let mut i = u * self.img.width() as f64;
        let mut j = v * self.img.height() as f64;

        if i >= self.img.width() as f64 {
            i = self.img.width() as f64 - 1.0;
        }

        if j >= self.img.height() as f64 {
            j = self.img.height() as f64 - 1.0;
        }

        let pixel = self.img.get_pixel(i as u32, j as u32);
        let color_scale = 1.0 / 255.0;
        Color::new(
            f64::from(pixel.0[0]) * color_scale,
            f64::from(pixel.0[1]) * color_scale,
            f64::from(pixel.0[2]) * color_scale,
        )
    }
}
