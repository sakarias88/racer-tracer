use crate::vec3::Color;

use super::ToneMap;

pub struct HableData {
    pub shoulder_strength: f64,
    pub linear_strength: f64,
    pub linear_angle: f64,
    pub toe_strength: f64,
    pub toe_numerator: f64,
    pub toe_denominator: f64,
}

pub struct Hable {
    data: HableData,
    toe_angle: f64,
    exposure_bias: f64,
    white_scale: f64,
}

impl Default for Hable {
    fn default() -> Self {
        let data = HableData {
            shoulder_strength: 0.15,
            linear_strength: 0.5,
            linear_angle: 0.1,
            toe_strength: 0.2,
            toe_numerator: 0.02,
            toe_denominator: 0.3,
        };
        let toe_angle = data.toe_numerator / data.toe_denominator;
        Self {
            white_scale: 1.0 / Hable::partial(&11.2, &data, toe_angle),
            toe_angle,
            exposure_bias: 2.0,
            data,
        }
    }
}

impl Hable {
    pub fn new(data: HableData, exposure_bias: f64, linear_white_point: f64) -> Self {
        let toe_angle = data.toe_numerator / data.toe_denominator;
        Self {
            white_scale: 1.0 / Hable::partial(&linear_white_point, &data, toe_angle),
            toe_angle,
            data,
            exposure_bias,
        }
    }

    fn partial(color: &f64, data: &HableData, toe_angle: f64) -> f64 {
        let a: f64 = data.shoulder_strength;
        let b: f64 = data.linear_strength;
        let c: f64 = data.linear_angle;
        let d: f64 = data.toe_strength;
        let e: f64 = data.toe_numerator;
        let f: f64 = data.toe_denominator;
        let e_div_f: f64 = toe_angle;
        ((color * (a * color + c * b) + d * e) / (color * (a * color + b) + d * f)) - e_div_f
    }

    fn hable(&self, color: Color) -> Color {
        Color::new(
            Hable::partial(color.x(), &self.data, self.toe_angle),
            Hable::partial(color.y(), &self.data, self.toe_angle),
            Hable::partial(color.z(), &self.data, self.toe_angle),
        )
    }
}

impl ToneMap for Hable {
    fn tone_map(&self, color: &Color) -> Color {
        let linear_color = self.hable(color * self.exposure_bias);
        Color::new(
            linear_color.x() * self.white_scale,
            linear_color.y() * self.white_scale,
            linear_color.z() * self.white_scale,
        )
    }
}
