use std::{path::PathBuf, str::FromStr};

use config::File;
use serde::Deserialize;
use structopt::StructOpt;

use crate::error::TracerError;

#[derive(Default, Debug, Deserialize)]
pub struct Screen {
    pub height: usize,
    pub width: usize,
}

#[derive(Default, Debug, Deserialize)]
pub struct RenderData {
    pub samples: usize,
    pub max_depth: usize,
    pub num_threads_width: usize,
    pub num_threads_height: usize,
    pub scale: usize,
}

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
    pub image_action: Option<ImageAction>,
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
                    cfg.loader = SceneLoader::Random;
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
                            "yml" => Ok(SceneLoader::Yml { path: path.clone() }),
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

#[derive(StructOpt, Debug, Clone, Deserialize, Default)]
pub enum SceneLoader {
    #[default]
    None,
    Yml {
        path: PathBuf,
    },
    Random,
}

#[derive(StructOpt, Debug, Clone, Deserialize, Default)]
pub enum ImageAction {
    #[default]
    WaitForSignal,
    SavePng,
}

impl FromStr for ImageAction {
    type Err = TracerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "png" => Ok(ImageAction::SavePng),
            "show" => Ok(ImageAction::WaitForSignal),
            _ => Ok(ImageAction::WaitForSignal),
        }
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub preview: RenderData,

    #[serde(default)]
    pub render: RenderData,

    #[serde(default)]
    pub screen: Screen,

    #[serde(default)]
    pub loader: SceneLoader,

    #[serde(default)]
    pub image_action: ImageAction,

    #[serde(default)]
    pub image_output_dir: Option<PathBuf>,
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
