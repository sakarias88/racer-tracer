use crate::vec3::Color;

use super::ToneMap;

pub struct Reinhard {
    max_white_pow: f64,
}

impl Reinhard {
    pub fn new(max_white: f64) -> Self {
        Reinhard {
            max_white_pow: max_white * max_white,
        }
    }

    fn luminance(color: &Color) -> f64 {
        color.dot(&Color::new(0.2126, 0.7152, 0.0722))
    }

    fn change_luminance(color: &Color, luminance: f64) -> Color {
        let color_luminance = Reinhard::luminance(color);
        color * (luminance / color_luminance)
    }
}

impl Default for Reinhard {
    fn default() -> Self {
        Self {
            max_white_pow: 25.0,
        }
    }
}

impl ToneMap for Reinhard {
    fn tone_map(&self, color: &Color) -> Color {
        let l_old = Reinhard::luminance(color);
        let numerator = l_old * (1.0 + (l_old / self.max_white_pow));
        let l_new = numerator / (1.0 + l_old);

        Reinhard::change_luminance(color, l_new)
    }
}
