use slog::Logger;

use crate::{config::Config, error::TracerError, vec3::Color};

use super::ImageAction;

pub struct None {}

impl ImageAction for None {
    fn action(
        &self,
        _screen_buffer: &[Color],
        _config: &Config,
        _log: &Logger,
    ) -> Result<(), TracerError> {
        Ok(())
    }
}
