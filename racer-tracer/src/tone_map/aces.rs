use crate::vec3::Color;

use super::ToneMap;

pub struct Aces {
    input_matrix: [Color; 3],
    output_matrix: [Color; 3],
}

impl Aces {
    pub fn new(input_matrix: [Color; 3], output_matrix: [Color; 3]) -> Self {
        Self {
            input_matrix,
            output_matrix,
        }
    }

    fn mul(matrix: [Color; 3], color: &Color) -> Color {
        let x = matrix[0][0] * color[0] + matrix[0][1] * color[1] + matrix[0][2] * color[2];
        let y = matrix[1][0] * color[0] + matrix[1][1] * color[1] + matrix[1][2] * color[2];
        let z = matrix[2][0] * color[0] + matrix[2][1] * color[1] + matrix[2][2] * color[2];
        Color::new(x, y, z)
    }

    fn rtt_and_odt_fit(color: Color) -> Color {
        let a = color * (color + 0.0245786) - 0.000090537;
        let b = color * (0.983729 * color + 0.4329510) + 0.238081;
        Color::new(a.x() / b.x(), a.y() / b.y(), a.z() / b.z())
    }
}

impl Default for Aces {
    fn default() -> Self {
        Self {
            input_matrix: [
                Color::new(0.59719, 0.35458, 0.04823),
                Color::new(0.07600, 0.90834, 0.01566),
                Color::new(0.02840, 0.13383, 0.83777),
            ],
            output_matrix: [
                Color::new(1.60475, -0.53108, -0.07367),
                Color::new(-0.10208, 1.10813, -0.00605),
                Color::new(-0.00327, -0.07276, 1.07602),
            ],
        }
    }
}

impl ToneMap for Aces {
    fn tone_map(&self, color: &Color) -> Color {
        Aces::mul(
            self.output_matrix,
            &Aces::rtt_and_odt_fit(Aces::mul(self.input_matrix, color)),
        )
    }
}
