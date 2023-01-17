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

// TODO: SubImage and Image can probably be the same struct
impl From<&Image> for SubImage {
    fn from(image: &Image) -> Self {
        SubImage {
            x: 0,
            y: 0,
            width: image.width,
            height: image.height,
            screen_width: image.width,
            screen_height: image.height,
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

pub trait QuadSplit {
    fn quad_split(&self) -> [SubImage; 4];
}

impl QuadSplit for SubImage {
    fn quad_split(&self) -> [SubImage; 4] {
        let half_w = self.width / 2;
        let half_h = self.height / 2;

        [
            // Top Left
            SubImage {
                x: self.x,
                y: self.y,
                width: half_w,
                height: half_h,
                screen_width: self.screen_width,
                screen_height: self.screen_height,
            },
            // Top Right
            SubImage {
                x: self.x + half_w,
                y: self.y,
                width: half_w,
                height: half_h,
                screen_width: self.screen_width,
                screen_height: self.screen_height,
            },
            // Bottom Left
            SubImage {
                x: self.x,
                y: self.y + half_h,
                width: half_w,
                height: half_h,
                screen_width: self.screen_width,
                screen_height: self.screen_height,
            },
            // Bottom Right
            SubImage {
                x: self.x + half_w,
                y: self.y + half_h,
                width: half_w,
                height: half_h,
                screen_width: self.screen_width,
                screen_height: self.screen_height,
            },
        ]
    }
}
