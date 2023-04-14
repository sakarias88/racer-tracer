pub mod interactive;

use slog::Logger;

use crate::{
    camera::Camera,
    config::Config,
    error::TracerError,
    image::Image,
    key_inputs::{KeyCallback, MouseCallback},
    scene::Scene,
    terminal::Terminal,
};

pub fn create_screen_buffer(image: &Image) -> Vec<u32> {
    vec![0; image.width * image.height]
}

pub struct SceneData {
    pub log: Logger,
    pub term: Terminal,
    pub config: Config,
    pub scene: Scene,
    pub camera: Camera,
    pub image: Image,
}

pub trait SceneController: Send + Sync {
    // Return a vector of key callbacks. The provided closure will be
    // called when the corresponding key is release/pressed.
    fn key_inputs(&self) -> Vec<KeyCallback>;

    fn mouse_input(&self) -> Option<MouseCallback>;

    // Render function
    fn render(&self) -> Result<(), TracerError>;

    // Returns the screen buffer produced by the scene controller.
    // Returns None if no new buffer is available
    fn get_buffer(&self) -> Result<Option<Vec<u32>>, TracerError>;

    // Called when the application wants to exit.
    fn stop(&self);
}
