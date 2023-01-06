#[derive(Clone)]
pub struct Image {
    pub aspect_ratio: f64,
    pub width: usize,
    pub height: usize,
}

impl Image {
    pub fn new(aspect_ratio: f64, width: usize) -> Image {
        Image {
            aspect_ratio,
            width,
            height: (width as f64 / aspect_ratio) as usize,
        }
    }
}
