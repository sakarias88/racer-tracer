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
    pub recurse_depth: usize,
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
}

impl TryFrom<Args> for Config {
    type Error = TracerError;
    fn try_from(args: Args) -> Result<Self, TracerError> {
        Config::from_file(args.config).map(|mut cfg| {
            if args.scene.is_some() {
                cfg.scene = args.scene;
            }
            cfg
        })
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
    pub scene: Option<String>,
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
