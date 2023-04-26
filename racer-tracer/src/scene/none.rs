use crate::{error::TracerError, scene::SceneLoader};

use super::SceneObject;

pub struct NoneLoader {}

impl NoneLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl SceneLoader for NoneLoader {
    fn load(&self) -> Result<Vec<SceneObject>, TracerError> {
        Ok(Vec::new())
    }
}
