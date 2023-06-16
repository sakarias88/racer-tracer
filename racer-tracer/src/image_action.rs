pub mod none;
pub mod png;

use slog::Logger;

use crate::image_action::{none::None, png::SavePng};

use crate::vec3::Color;
use crate::{
    config::{Config, ImageActionConfig},
    error::TracerError,
};

pub trait ImageAction: Send + Sync {
    fn action(
        &self,
        screen_buffer: &[Color],
        config: &Config,
        log: &Logger,
    ) -> Result<(), TracerError>;
}

impl From<&ImageActionConfig> for &dyn ImageAction {
    fn from(image_action: &ImageActionConfig) -> Self {
        match image_action {
            ImageActionConfig::None => &None {} as &dyn ImageAction,
            ImageActionConfig::SavePng => &SavePng {} as &dyn ImageAction,
        }
    }
}
