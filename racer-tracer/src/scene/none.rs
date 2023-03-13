use crate::{error::TracerError, scene::SceneLoader};

pub struct NoneLoader {}

impl NoneLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl SceneLoader for NoneLoader {
    fn load(&self) -> Result<Vec<Box<dyn crate::geometry::Hittable>>, TracerError> {
        Ok(Vec::new())
    }
}
