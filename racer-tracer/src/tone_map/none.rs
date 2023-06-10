use crate::vec3::Color;

use super::ToneMap;

pub struct None;

impl None {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

impl ToneMap for None {
    fn tone_map(&self, color: Color) -> crate::vec3::Color {
        color
    }
}
