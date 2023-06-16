use std::sync::RwLock;

use slog::Logger;
use synchronoise::SignalEvent;

use crate::{config::Config, error::TracerError, terminal::Terminal};

use super::ImageAction;

pub struct None {}

impl ImageAction for None {
    fn action(
        &self,
        _screen_buffer: &RwLock<Vec<u32>>,
        _cancel_event: &SignalEvent,
        _event: &SignalEvent,
        _config: &Config,
        _log: Logger,
        _term: &Terminal,
    ) -> Result<(), TracerError> {
        Ok(())
    }
}
