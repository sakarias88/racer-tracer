pub mod interactive;

use crate::{
    background_color::BackgroundColor,
    camera::{Camera, SharedCamera},
    error::TracerError,
    geometry::Hittable,
    image::Image,
    key_inputs::{KeyEvent, ListenKeyEvents, MousePos},
    scene::Scene,
};

pub fn create_screen_buffer(image: &Image) -> Vec<u32> {
    vec![0; image.width * image.height]
}

pub trait SceneController: Send + Sync {
    fn update(
        &self,
        dt: f64,
        keys: Vec<KeyEvent>,
        mouse_pos: Option<MousePos>,
        camera: &mut Camera,
        scene: &mut Scene,
    ) -> Result<(), TracerError>;
    fn register_key_inputs(&self) -> Vec<ListenKeyEvents>;

    // Render function
    fn render(
        &self,
        scene_changed: bool,
        camera: &SharedCamera,
        scene: &dyn Hittable,
        background: &dyn BackgroundColor,
    ) -> Result<(), TracerError>;

    // Returns the screen buffer produced by the scene controller.
    // Returns None if no new buffer is available
    fn get_buffer(&self) -> Result<Option<Vec<u32>>, TracerError>;

    // Called when the application wants to exit.
    fn stop(&self);
}

pub trait SceneRenderer: Send + Sync {}
