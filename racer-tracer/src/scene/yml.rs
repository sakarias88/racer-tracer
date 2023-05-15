// TODO: Add support for textures and other geometry. Can't be
// bothered to keep the parsing code up to date while developing
// things that might change.
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;

use crate::{
    background_color::{BackgroundColor, Sky, SolidBackgroundColor},
    camera::CameraLoadData,
    error::TracerError,
    geometry_creation::create_sphere,
    material::{dialectric::Dialectric, lambertian::Lambertian, metal::Metal, Material},
    scene::SceneLoader,
    vec3::{Color, Vec3},
};

use config::File;

use super::{SceneLoadData, SceneObject};

pub struct YmlLoader {
    path: PathBuf,
}

impl YmlLoader {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl SceneLoader for YmlLoader {
    fn load(&self) -> Result<SceneLoadData, TracerError> {
        SceneData::from_file(PathBuf::from(&self.path)).and_then(|data| data.try_into())
    }
}

#[derive(Debug, Deserialize)]
enum MaterialData {
    Lambertian { color: Color },
    Metal { color: Color, fuzz: f64 },
    Dialectric { refraction_index: f64 },
}

#[derive(Debug, Deserialize)]
enum GeometryData {
    Sphere {
        pos: Vec3,
        radius: f64,
        material: String,
    },
}

#[derive(Debug, Deserialize)]
enum Background {
    Sky { top: Vec3, bottom: Vec3 },
    SolidColor(Vec3),
}

#[derive(Deserialize)]
struct SceneData {
    materials: HashMap<String, MaterialData>,
    geometry: Vec<GeometryData>,
    background: Option<Background>,
    camera: Option<CameraLoadData>,
}

impl SceneData {
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self, TracerError> {
        config::Config::builder()
            .add_source(File::from(file.as_ref()))
            .build()
            .map_err(|e| {
                TracerError::Configuration(
                    file.as_ref().to_string_lossy().into_owned(),
                    dbg!(e).to_string(),
                )
            })?
            .try_deserialize()
            .map_err(|e| {
                TracerError::Configuration(
                    file.as_ref().to_string_lossy().into_owned(),
                    dbg!(e).to_string(),
                )
            })
    }
}

impl TryInto<SceneLoadData> for SceneData {
    type Error = TracerError;
    fn try_into(self) -> Result<SceneLoadData, TracerError> {
        let mut materials: HashMap<String, Arc<dyn Material>> = HashMap::new();
        self.materials
            .into_iter()
            .for_each(|(id, material)| match material {
                MaterialData::Lambertian { color } => {
                    materials.insert(id, Arc::new(Lambertian::new_with_color(color)));
                }
                MaterialData::Metal { color, fuzz } => {
                    materials.insert(id, Arc::new(Metal::new_with_color(color, fuzz)));
                }
                MaterialData::Dialectric { refraction_index } => {
                    materials.insert(id, Arc::new(Dialectric::new(refraction_index)));
                }
            });

        let geometry: Vec<SceneObject> = self
            .geometry
            .into_iter()
            .map(|geo| match geo {
                GeometryData::Sphere {
                    pos,
                    radius,
                    material,
                } => materials
                    .get(&material)
                    .ok_or(TracerError::UnknownMaterial(material))
                    .map(|mat| create_sphere(Arc::clone(mat), pos, radius)),
            })
            .collect::<Result<Vec<SceneObject>, TracerError>>()?;
        Ok(SceneLoadData {
            objects: geometry,
            background: match self.background {
                Some(v) => match v {
                    Background::Sky { top, bottom } => {
                        Box::new(Sky::new(top, bottom)) as Box<dyn BackgroundColor>
                    }
                    Background::SolidColor(color) => {
                        Box::new(SolidBackgroundColor::new(color)) as Box<dyn BackgroundColor>
                    }
                },
                None => Box::<Sky>::default() as Box<dyn BackgroundColor>,
            },
            camera: self.camera,
        })
    }
}
