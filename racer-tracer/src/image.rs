#[derive(Clone)]
pub struct Image {
    pub aspect_ratio: f64,
    pub width: usize,
    pub height: usize,
    pub samples_per_pixel: usize,
}

impl Image {
    pub fn new(aspect_ratio: f64, width: usize, samples_per_pixel: usize) -> Image {
        Image {
            aspect_ratio,
            width,
            height: (width as f64 / aspect_ratio) as usize,
            samples_per_pixel,
        }
    }
}
