use crate::data_bus::DataWriter;

#[derive(Clone)]
pub struct Image {
    pub aspect_ratio: f64,
    pub width: usize,
    pub height: usize,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        Image {
            aspect_ratio: width as f64 / height as f64,
            width,
            height,
        }
    }
}

impl Image {
    pub fn screen_to_uv(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        (screen_x / self.width as f64, screen_y / self.height as f64)
    }
}

pub struct SubImage<T: Clone> {
    pub x: usize,
    pub y: usize,
    pub screen_width: usize,
    pub screen_height: usize,
    pub width: usize,
    pub height: usize,
    pub writer: DataWriter<T>,
}
