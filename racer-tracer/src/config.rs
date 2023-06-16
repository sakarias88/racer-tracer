use std::{path::PathBuf, str::FromStr};

use config::File;
use serde::Deserialize;
use structopt::StructOpt;

use crate::{
    error::TracerError,
    vec3::{Color, Vec3},
};

#[derive(StructOpt, Debug)]
#[structopt(name = "racer-tracer")]
pub struct Args {
    #[structopt(
        short = "c",
        long = "config",
        default_value = "./config.yml",
        env = "CONFIG"
    )]
    pub config: String,

    #[structopt(short = "s", long = "scene")]
    pub scene: Option<String>,

    #[structopt(long = "image-action")]
    pub image_action: Option<ImageActionConfig>,
}

impl TryFrom<Args> for Config {
    type Error = TracerError;
    fn try_from(args: Args) -> Result<Self, TracerError> {
        Config::from_file(args.config).and_then(|mut cfg| {
            if let Some(image_action) = args.image_action {
                cfg.image_action = image_action;
            }

            if let Some(scene) = args.scene {
                if scene == "random" {
                    cfg.loader = SceneLoaderConfig::Random;
                } else if scene == "sandbox" {
                    cfg.loader = SceneLoaderConfig::Sandbox;
                } else {
                    let path = PathBuf::from(scene);
                    cfg.loader = path
                        .extension()
                        .map(|s| s.to_string_lossy())
                        .ok_or_else(|| {
                            TracerError::ArgumentParsingError(format!(
                                "Could not get extension from scene file: {}",
                                path.display()
                            ))
                        })
                        .and_then(|p| match p.as_ref() {
                            "yml" => Ok(SceneLoaderConfig::Yml { path: path.clone() }),
                            _ => Err(TracerError::ArgumentParsingError(format!(
                                "Could not find a suitable scene loader for file: {}",
                                path.display()
                            ))),
                        })?;
                };
            }

            Ok(cfg)
        })
    }
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct ScreenConfig {
    pub height: usize,
    pub width: usize,
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct RenderConfig {
    pub samples: usize,
    pub max_depth: usize,
    pub num_threads_width: usize,
    pub num_threads_height: usize,
    pub scale: usize,
}

#[derive(StructOpt, Debug, Clone, Deserialize, Default)]
pub enum SceneLoaderConfig {
    #[default]
    None,
    Yml {
        path: PathBuf,
    },
    Random,
    Sandbox,
}

#[derive(StructOpt, Debug, Clone, Deserialize, Default)]
pub enum ImageActionConfig {
    #[default]
    None,
    SavePng,
}

#[derive(StructOpt, Debug, Clone, Deserialize, Default)]
pub enum SceneControllerConfig {
    #[default]
    Interactive,
}

#[derive(StructOpt, Debug, Clone, Deserialize, Default)]
pub enum RendererConfig {
    #[default]
    Cpu,
    CpuPreview,
}

fn default_preview() -> RendererConfig {
    RendererConfig::CpuPreview
}

impl FromStr for ImageActionConfig {
    type Err = TracerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "png" => Ok(ImageActionConfig::SavePng),
            "none" => Ok(ImageActionConfig::None),
            _ => Ok(ImageActionConfig::None),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorMatrix {
    pub colors: [Color; 3],
}

// The config crate can't create an enum even if all fields have
// defaults. It can however create the enum as long as you provide at
// least one field. This is why we see the default field on some of
// them. It's just a junk field to avoid the bug and so you can select
// a tone mapping technique without having to override any default
// settings.
// https://github.com/mehcode/config-rs/issues/126
#[derive(Default, Debug, Clone, Deserialize)]
pub enum ToneMapConfig {
    Reinhard {
        default: Option<bool>,
        max_white: Option<f64>,
    },
    Hable {
        default: Option<bool>,
        shoulder_strength: Option<f64>,
        linear_strength: Option<f64>,
        linear_angle: Option<f64>,
        toe_strength: Option<f64>,
        toe_numerator: Option<f64>,
        toe_denominator: Option<f64>,
        exposure_bias: Option<f64>,
        linear_white_point: Option<f64>,
    },
    Aces {
        default: Option<bool>,
        input_matrix: Option<ColorMatrix>,
        output_matrix: Option<ColorMatrix>,
    },
    #[default]
    None,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct CameraConfig {
    pub vfov: Option<f64>,
    pub aperture: Option<f64>,
    pub focus_distance: Option<f64>,
    pub pos: Option<Vec3>,
    pub look_at: Option<Vec3>,
    pub speed: Option<f64>,
    pub sensitivity: Option<f64>,
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub preview: RenderConfig,

    #[serde(default)]
    pub render: RenderConfig,

    #[serde(default)]
    pub screen: ScreenConfig,

    #[serde(default)]
    pub loader: SceneLoaderConfig,

    #[serde(default)]
    pub image_action: ImageActionConfig,

    #[serde(default)]
    pub image_output_dir: Option<PathBuf>,

    #[serde(default)]
    pub scene_controller: SceneControllerConfig,

    #[serde(default)]
    pub renderer: RendererConfig,

    #[serde(default = "default_preview")]
    pub preview_renderer: RendererConfig,

    #[serde(default)]
    pub camera: CameraConfig,

    #[serde(default)]
    pub tone_map: ToneMapConfig,
}

impl Config {
    pub fn from_file(file: String) -> Result<Self, TracerError> {
        config::Config::builder()
            .add_source(File::from(file.as_ref()))
            .build()
            .map_err(|e| TracerError::Configuration(file.clone(), e.to_string()))?
            .try_deserialize()
            .map_err(|e| TracerError::Configuration(file, e.to_string()))
    }
}
