use std::{collections::HashMap, path::Path, sync::Arc};

use config::File;
use serde::Deserialize;

use crate::{
    error::TracerError,
    geometry::{sphere::Sphere, Hittable},
    material::{dialectric::Dialectric, lambertian::Lambertian, metal::Metal, SharedMaterial},
    vec3::{Color, Vec3},
};

pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, hittable: Box<dyn Hittable>) {
        self.objects.push(hittable);
    }

    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self, TracerError> {
        SceneData::from_file(file)?.try_into()
    }
}

impl Hittable for Scene {
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
    ) -> Option<crate::geometry::HitRecord> {
        let mut rec = None;
        let mut closes_so_far = t_max;

        for obj in self.objects.iter() {
            if let Some(hit_rec) = obj.hit(ray, t_min, closes_so_far) {
                closes_so_far = hit_rec.t;
                rec = Some(hit_rec);
            }
        }

        rec
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
                    e.to_string(),
                )
            })?
            .try_deserialize()
            .map_err(|e| {
                TracerError::Configuration(
                    file.as_ref().to_string_lossy().into_owned(),
                    e.to_string(),
                )
            })
    }
}

impl TryInto<Scene> for SceneData {
    type Error = TracerError;
    fn try_into(self) -> Result<Scene, TracerError> {
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
                        let apa: Box<dyn Hittable> =
                            Box::new(Sphere::new(pos, radius, Arc::clone(mat)));
                        apa
                    }),
            })
            .collect::<Result<Vec<Box<dyn Hittable>>, TracerError>>()?;

        Ok(Scene { objects: geometry })
    }
}
