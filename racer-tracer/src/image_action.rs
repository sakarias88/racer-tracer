use std::{path::PathBuf, sync::RwLock};

use sha2::{Digest, Sha256};
use synchronoise::SignalEvent;

use crate::{
    config::{Config, ImageAction as CImageAction},
    error::TracerError,
};

pub trait ImageAction: Send + Sync {
    fn action(
        &self,
        screen_buffer: &RwLock<Vec<u32>>,
        event: &SignalEvent,
        config: &Config,
    ) -> Result<(), TracerError>;
}

impl From<&CImageAction> for Box<dyn ImageAction> {
    fn from(image_action: &CImageAction) -> Self {
        match image_action {
            CImageAction::WaitForSignal => Box::new(WaitForSignal::new()),
            CImageAction::SavePng => Box::new(SavePng::new()),
        }
    }
}

pub struct SavePng {}

impl SavePng {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageAction for SavePng {
    fn action(
        &self,
        screen_buffer: &RwLock<Vec<u32>>,
        _event: &SignalEvent,
        config: &Config,
    ) -> Result<(), TracerError> {
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
                    println!("Saving image...");
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
                        println!("Saved image to: {}", file_path.to_string_lossy());
                    })
                }
                None => Ok(()),
            })
    }
}

pub struct WaitForSignal {}

impl WaitForSignal {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageAction for WaitForSignal {
    fn action(
        &self,
        _screen_buffer: &RwLock<Vec<u32>>,
        event: &SignalEvent,
        _config: &Config,
    ) -> Result<(), TracerError> {
        println!("Press R to resume.");
        event.wait();
        event.reset();
        Ok(())
    }
}
