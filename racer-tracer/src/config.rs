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
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub preview: RenderData,

    #[serde(default)]
    pub render: RenderData,

    #[serde(default)]
    pub screen: Screen,
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
