pub mod interactive;

use synchronoise::SignalEvent;

use crate::{
    background_color::BackgroundColor,
    camera::{Camera, SharedCamera},
    error::TracerError,
    geometry::Hittable,
    image_buffer::ImageBufferWriter,
    key_inputs::{KeyEvent, ListenKeyEvents, MousePos},
    scene::Scene,
};

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
        image_buffer_writer: &ImageBufferWriter,
        rendered_image_completed: &SignalEvent,
    ) -> Result<(), TracerError>;

    // Called when the application wants to exit.
    fn stop(&self);
}

pub trait SceneRenderer: Send + Sync {}
