pub mod png;
pub mod wait_for_signal;

use std::sync::RwLock;

use slog::Logger;
use synchronoise::SignalEvent;

use crate::image_action::{png::SavePng, wait_for_signal::WaitForSignal};

use crate::terminal::Terminal;
use crate::{
    config::{Config, ImageActionConfig},
    error::TracerError,
};

pub trait ImageAction: Send + Sync {
    fn action(
        &self,
        screen_buffer: &RwLock<Vec<u32>>,
        cancel_event: &SignalEvent,
        event: &SignalEvent,
        config: &Config,
        log: Logger,
        term: &Terminal,
    ) -> Result<(), TracerError>;
}

impl From<&ImageActionConfig> for &dyn ImageAction {
    fn from(image_action: &ImageActionConfig) -> Self {
        match image_action {
            ImageActionConfig::WaitForSignal => &WaitForSignal {} as &dyn ImageAction,
            ImageActionConfig::SavePng => &SavePng {} as &dyn ImageAction,
        }
    }
}
