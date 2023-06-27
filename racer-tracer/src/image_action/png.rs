use std::path::PathBuf;

use sha2::{Digest, Sha256};
use slog::Logger;

use crate::{config::Config, error::TracerError, vec3::Color};

use super::ImageAction;

pub struct SavePng {}

impl ImageAction for SavePng {
    fn action(
        &self,
        screen_buffer: &[Color],
        config: &Config,
        log: &Logger,
    ) -> Result<(), TracerError> {
        match &config.image_output_dir {
            Some(image_dir) => {
                let png_data = screen_buffer
                    .iter()
                    .map(|v| {
                        let red: u32 = (v[0] * 255.0) as u32;
                        let green: u32 = (v[1] * 255.0) as u32;
                        let blue: u32 = (v[2] * 255.0) as u32;
                        // RGBA
                        (red << 24) | green << 16 | blue << 8 | 255
                    })
                    .flat_map(|val| val.to_be_bytes())
                    .collect::<Vec<u8>>();

                info!(log, "Saving image...");
                let mut sha = Sha256::new();

                sha.update(&png_data);

                let mut file_path = PathBuf::from(image_dir);
                file_path.push(format!("{:X}.png", sha.finalize()));

                img::save_buffer(
                    file_path.as_path(),
                    png_data.as_slice(),
                    config.screen.width as u32,
                    config.screen.height as u32,
                    img::ColorType::Rgba8,
                )
                .map_err(|e| {
                    let error = e.to_string();
                    TracerError::ImageSave(error)
                })
                .map(|_| {
                    info!(log, "Saved image to: {}", file_path.to_string_lossy());
                })
            }
            None => {
                info!(log, "No output directory for saving pngs. Skipping.");
                Ok(())
            }
        }
    }
}
