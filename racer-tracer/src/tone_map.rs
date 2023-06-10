pub mod aces;
pub mod hable;
pub mod none;
pub mod reinhard;

use crate::config::ToneMapConfig;
use crate::vec3::Color;

use self::aces::Aces;
use self::hable::{Hable, HableData};
use self::none::None;
use self::reinhard::Reinhard;

pub trait ToneMap: Send + Sync {
    fn tone_map(&self, color: Color) -> Color;
}

impl From<&ToneMapConfig> for Box<dyn ToneMap> {
    fn from(tone_mapping: &ToneMapConfig) -> Self {
        match tone_mapping {
            ToneMapConfig::Reinhard { max_white, .. } => {
                Box::new(Reinhard::new(max_white.unwrap_or(25.0))) as Box<dyn ToneMap>
            }
            ToneMapConfig::Hable {
                shoulder_strength,
                linear_strength,
                linear_angle,
                toe_strength,
                toe_numerator,
                toe_denominator,
                exposure_bias,
                linear_white_point,
                ..
            } => Box::new(Hable::new(
                HableData {
                    shoulder_strength: shoulder_strength.unwrap_or(0.15),
                    linear_strength: linear_strength.unwrap_or(0.5),
                    linear_angle: linear_angle.unwrap_or(0.1),
                    toe_strength: toe_strength.unwrap_or(0.2),
                    toe_numerator: toe_numerator.unwrap_or(0.02),
                    toe_denominator: toe_denominator.unwrap_or(0.3),
                },
                exposure_bias.unwrap_or(2.0),
                linear_white_point.unwrap_or(11.2),
            )) as Box<dyn ToneMap>,
            ToneMapConfig::Aces {
                input_matrix,
                output_matrix,
                ..
            } => {
                let input = input_matrix.as_ref().map(|m| m.colors).unwrap_or([
                    Color::new(0.59719, 0.35458, 0.04823),
                    Color::new(0.07600, 0.90834, 0.01566),
                    Color::new(0.02840, 0.13383, 0.83777),
                ]);
                let output = output_matrix.as_ref().map(|m| m.colors).unwrap_or([
                    Color::new(1.60475, -0.53108, -0.07367),
                    Color::new(-0.10208, 1.10813, -0.00605),
                    Color::new(-0.00327, -0.07276, 1.07602),
                ]);
                Box::new(Aces::new(input, output)) as Box<dyn ToneMap>
            }
            ToneMapConfig::None => Box::new(None::new()) as Box<dyn ToneMap>,
        }
    }
}
