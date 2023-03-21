use std::sync::RwLock;

use slog::Logger;
use synchronoise::SignalEvent;

use crate::{
    config::Config,
    error::TracerError,
    terminal::{write_term, Terminal},
};

use super::ImageAction;

pub struct WaitForSignal {}

impl ImageAction for WaitForSignal {
    fn action(
        &self,
        _screen_buffer: &RwLock<Vec<u32>>,
        event: &SignalEvent,
        _config: &Config,
        _log: Logger,
        term: &Terminal,
    ) -> Result<(), TracerError> {
        if !event.status() {
            write_term!(term, "Press R to resume.");
            event.wait();
        }
        event.reset();
        Ok(())
    }
}
