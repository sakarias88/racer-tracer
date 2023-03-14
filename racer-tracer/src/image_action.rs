pub mod png;
pub mod wait_for_signal;

use std::sync::RwLock;

use synchronoise::SignalEvent;

use crate::image_action::{png::SavePng, wait_for_signal::WaitForSignal};

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
            CImageAction::WaitForSignal => Box::new(WaitForSignal {}),
            CImageAction::SavePng => Box::new(SavePng {}),
        }
    }
}
