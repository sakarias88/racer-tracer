use crate::{background_color::Sky, error::TracerError, scene::SceneLoader};

use super::SceneLoadData;

pub struct NoneLoader {}

impl NoneLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl SceneLoader for NoneLoader {
    fn load(&self) -> Result<SceneLoadData, TracerError> {
        Ok(SceneLoadData {
            objects: Vec::new(),
            background: Box::<Sky>::default(),
            camera: None,
            tone_map: None,
        })
    }
}
