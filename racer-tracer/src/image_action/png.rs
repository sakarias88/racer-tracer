use std::{path::PathBuf, sync::RwLock};

use sha2::{Digest, Sha256};
use slog::Logger;
use synchronoise::SignalEvent;

use crate::{config::Config, error::TracerError, terminal::Terminal};

use super::ImageAction;

pub struct SavePng {}

impl ImageAction for SavePng {
    fn action(
        &self,
        screen_buffer: &RwLock<Vec<u32>>,
        event: &SignalEvent,
        config: &Config,
        log: Logger,
        _term: &Terminal,
    ) -> Result<(), TracerError> {
        let status = event.status();
        event.reset();
        if status {
            Ok(())
        } else {
            screen_buffer
                .read()
                .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
                .map(|buf| {
                    // Convert ARGB8 to RGBA8
                    buf.iter()
                        .map(|v| {
                            let a: u32 = (v >> 24) & 0xff;
                            let r: u32 = (v >> 16) & 0xff;
                            let g: u32 = (v >> 8) & 0xff;
                            let b: u32 = v & 0xff;

                            (r << 24) | (g << 16) | (b << 8) | a
                        })
                        .flat_map(|val| val.to_be_bytes())
                        .collect::<Vec<u8>>()
                })
                .and_then(|buf| match &config.image_output_dir {
                    Some(image_dir) => {
                        info!(log, "Saving image...");
                        let mut sha = Sha256::new();

                        sha.update(buf.as_slice());

                        let mut file_path = PathBuf::from(image_dir);
                        file_path.push(format!("{:X}.png", sha.finalize()));

                        img::save_buffer(
                            file_path.as_path(),
                            buf.as_slice(),
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
                    None => Ok(()),
                })
        }
    }
}
