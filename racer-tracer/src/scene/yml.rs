use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;

use crate::{
    error::TracerError,
    geometry::{sphere::Sphere, Hittable},
    material::{dialectric::Dialectric, lambertian::Lambertian, metal::Metal, SharedMaterial},
    scene::SceneLoader,
    vec3::{Color, Vec3},
};

use config::File;

pub struct YmlLoader {
    path: PathBuf,
}

impl YmlLoader {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl SceneLoader for YmlLoader {
    fn load(&self) -> Result<Vec<Box<dyn crate::geometry::Hittable>>, TracerError> {
        let datta = SceneData::from_file(PathBuf::from(&self.path))?;
        datta.try_into()
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

#[derive(Deserialize)]
struct SceneData {
    materials: HashMap<String, MaterialData>,
    geometry: Vec<GeometryData>,
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

impl TryInto<Vec<Box<dyn Hittable>>> for SceneData {
    type Error = TracerError;
    fn try_into(self) -> Result<Vec<Box<dyn Hittable>>, TracerError> {
        let mut materials: HashMap<String, SharedMaterial> = HashMap::new();
        self.materials
            .into_iter()
            .for_each(|(id, material)| match material {
                MaterialData::Lambertian { color } => {
                    materials.insert(id, Arc::new(Box::new(Lambertian::new(color))));
                }
                MaterialData::Metal { color, fuzz } => {
                    materials.insert(id, Arc::new(Box::new(Metal::new(color, fuzz))));
                }
                MaterialData::Dialectric { refraction_index } => {
                    materials.insert(id, Arc::new(Box::new(Dialectric::new(refraction_index))));
                }
            });

        let geometry: Vec<Box<dyn Hittable>> = self
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
                    .map(|mat| {
                        Box::new(Sphere::new(pos, radius, Arc::clone(mat))) as Box<dyn Hittable>
                    }),
            })
            .collect::<Result<Vec<Box<dyn Hittable>>, TracerError>>()?;

        Ok(geometry)
    }
}
