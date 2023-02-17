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

pub struct SubImage {
    pub x: usize,
    pub y: usize,
    pub screen_width: usize,
    pub screen_height: usize,
    pub width: usize,
    pub height: usize,
}
