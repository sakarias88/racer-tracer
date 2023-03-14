use std::sync::RwLock;

use synchronoise::SignalEvent;

use crate::{config::Config, error::TracerError};

use super::ImageAction;

pub struct WaitForSignal {}

impl ImageAction for WaitForSignal {
    fn action(
        &self,
        _screen_buffer: &RwLock<Vec<u32>>,
        event: &SignalEvent,
        _config: &Config,
    ) -> Result<(), TracerError> {
        if !event.status() {
            println!("Press R to resume.");
            event.wait();
        }
        event.reset();
        Ok(())
    }
}
