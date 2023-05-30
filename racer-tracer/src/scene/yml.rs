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
    geometry_creation::{
        create_box, create_rotate_y, create_sphere, create_translate, create_xy_rect,
        create_xz_rect, create_yz_rect,
    },
    material::{
        dialectric::Dialectric, diffuse_light::DiffuseLight, lambertian::Lambertian, metal::Metal,
        Material,
    },
    scene::SceneLoader,
    texture::{
        checkered::Checkered, image::TextureImage, noise::Noise, solid_color::SolidColor, Texture,
    },
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
enum TextureData {
    Checkered {
        texture_a: String,
        texture_b: String,
    },
    Image {
        path: PathBuf,
    },
    Noise {
        scale: f64,
        depth: i32,
        color: Color,
    },
    SolidColor {
        color: Color,
    },
}

#[derive(Debug, Deserialize)]
enum MaterialData {
    Lambertian {
        #[serde(alias = "texture")]
        texture_key: String,
    },
    Metal {
        #[serde(alias = "texture")]
        texture_key: String,
        fuzz: f64,
    },
    Dialectric {
        refraction_index: f64,
    },
    DiffuseLight {
        #[serde(alias = "texture")]
        texture_key: String,
    },
}

#[derive(Debug, Deserialize)]
enum GeometryData {
    Sphere {
        #[serde(flatten)]
        pos: Vec3,
        radius: f64,
        material: String,
    },
    XyRect {
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        material: String,
    },
    XzRect {
        x0: f64,
        x1: f64,
        z0: f64,
        z1: f64,
        k: f64,
        material: String,
    },
    YzRect {
        y0: f64,
        y1: f64,
        z0: f64,
        z1: f64,
        k: f64,
        material: String,
    },
    Box {
        min: Vec3,
        max: Vec3,
        material: String,
    },
    RotateY {
        key: String,
        degrees: f64,
    },
    Translate {
        key: String,
        #[serde(flatten)]
        pos: Vec3,
    },
}

#[derive(Debug, Deserialize)]
enum Background {
    Sky { top: Vec3, bottom: Vec3 },
    SolidColor(Vec3),
}

#[derive(Deserialize)]
struct SceneData {
    textures: HashMap<String, TextureData>,
    materials: HashMap<String, MaterialData>,
    geometry: HashMap<String, GeometryData>,
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
        let mut textures: HashMap<String, Arc<dyn Texture>> = HashMap::new();
        let mut checkered_textures: HashMap<String, TextureData> = HashMap::new();

        self.textures
            .into_iter()
            .try_for_each::<_, Result<(), TracerError>>(|(key, texture)| {
                match texture {
                    TextureData::Checkered {
                        texture_a,
                        texture_b,
                    } => {
                        checkered_textures.insert(
                            key,
                            TextureData::Checkered {
                                texture_a,
                                texture_b,
                            },
                        );
                    }
                    TextureData::Image { path } => {
                        textures.insert(key, Arc::new(TextureImage::try_new(&path)?));
                    }
                    TextureData::Noise {
                        scale,
                        depth,
                        color,
                    } => {
                        textures.insert(key, Arc::new(Noise::new(scale, Some(depth), color)));
                    }
                    TextureData::SolidColor { color } => {
                        textures.insert(key, Arc::new(SolidColor::new(color)));
                    }
                }
                Ok(())
            })?;

        // Index checkered textures afterwards since they depend on
        // other textures existing.
        checkered_textures
            .into_iter()
            .try_for_each(|(key, check)| match check {
                TextureData::Checkered {
                    texture_a,
                    texture_b,
                } => {
                    let tex_a = textures.get(&texture_a).ok_or_else(|| {
                        TracerError::SceneLoad(format!(
                            "Checkered texture \"{}\" expected texture \"{}\" to exist.",
                            key, texture_a
                        ))
                    })?;
                    let tex_b = textures.get(&texture_b).ok_or_else(|| {
                        TracerError::SceneLoad(format!(
                            "Checkered texture \"{}\" expected texture \"{}\" to exist.",
                            key, texture_a
                        ))
                    })?;
                    textures.insert(
                        key,
                        Arc::new(Checkered::new(Arc::clone(tex_a), Arc::clone(tex_b))),
                    );
                    Ok(())
                }
                _ => Err(TracerError::SceneLoad(format!(
                    "Expected texture \"{}\" to be of the checkered variant.",
                    key
                ))),
            })?;

        let mut materials: HashMap<String, Arc<dyn Material>> = HashMap::new();
        self.materials
            .into_iter()
            .try_for_each::<_, Result<(), TracerError>>(|(key, material)| match material {
                MaterialData::Lambertian { texture_key } => textures
                    .get(&texture_key)
                    .ok_or_else(|| {
                        TracerError::SceneLoad(format!(
                            "Failed to find texture \"{}\" for lambertian material \"{}\"",
                            texture_key, key
                        ))
                    })
                    .map(|texture| {
                        materials.insert(key, Arc::new(Lambertian::new(Arc::clone(texture))));
                    }),
                MaterialData::Metal { texture_key, fuzz } => textures
                    .get(&texture_key)
                    .ok_or_else(|| {
                        TracerError::SceneLoad(format!(
                            "Failed to find texture \"{}\" for metal material \"{}\"",
                            texture_key, key
                        ))
                    })
                    .map(|texture| {
                        materials.insert(key, Arc::new(Metal::new(Arc::clone(texture), fuzz)));
                    }),
                MaterialData::Dialectric { refraction_index } => {
                    materials.insert(key, Arc::new(Dialectric::new(refraction_index)));
                    Ok(())
                }
                MaterialData::DiffuseLight { texture_key } => textures
                    .get(&texture_key)
                    .ok_or_else(|| {
                        TracerError::SceneLoad(format!(
                            "Failed to find texture \"{}\" for diffuse light material \"{}\"",
                            texture_key, key
                        ))
                    })
                    .map(|texture| {
                        materials.insert(key, Arc::new(DiffuseLight::new(Arc::clone(texture))));
                    }),
            })?;

        let mut rotations_y: HashMap<String, GeometryData> = HashMap::new();
        let mut translations: HashMap<String, GeometryData> = HashMap::new();
        let mut geometry: HashMap<String, SceneObject> = HashMap::new();

        self.geometry
            .into_iter()
            .try_for_each::<_, Result<(), TracerError>>(|(key, geo)| match geo {
                GeometryData::Sphere {
                    pos,
                    radius,
                    material,
                } => materials
                    .get(&material)
                    .ok_or(TracerError::UnknownMaterial(material))
                    .and_then(|mat| {
                        match geometry
                            .insert(key.clone(), create_sphere(Arc::clone(mat), pos, radius))
                        {
                            Some(_) => Err(TracerError::SceneLoad(format!(
                                "The object \"{}\" was already present in the scene.",
                                key
                            ))),
                            None => Ok(()),
                        }
                    }),
                GeometryData::XyRect {
                    x0,
                    x1,
                    y0,
                    y1,
                    k,
                    material,
                } => materials
                    .get(&material)
                    .ok_or(TracerError::UnknownMaterial(material))
                    .and_then(|mat| {
                        match geometry.insert(
                            key.clone(),
                            create_xy_rect(Arc::clone(mat), x0, x1, y0, y1, k),
                        ) {
                            Some(_) => Err(TracerError::SceneLoad(format!(
                                "The object \"{}\" was already present in the scene.",
                                key
                            ))),
                            None => Ok(()),
                        }
                    }),
                GeometryData::XzRect {
                    x0,
                    x1,
                    z0,
                    z1,
                    k,
                    material,
                } => materials
                    .get(&material)
                    .ok_or(TracerError::UnknownMaterial(material))
                    .and_then(|mat| {
                        match geometry.insert(
                            key.clone(),
                            create_xz_rect(Arc::clone(mat), x0, x1, z0, z1, k),
                        ) {
                            Some(_) => Err(TracerError::SceneLoad(format!(
                                "The object \"{}\" was already present in the scene.",
                                key
                            ))),
                            None => Ok(()),
                        }
                    }),
                GeometryData::YzRect {
                    y0,
                    y1,
                    z0,
                    z1,
                    k,
                    material,
                } => materials
                    .get(&material)
                    .ok_or(TracerError::UnknownMaterial(material))
                    .and_then(|mat| {
                        match geometry.insert(
                            key.clone(),
                            create_yz_rect(Arc::clone(mat), y0, y1, z0, z1, k),
                        ) {
                            Some(_) => Err(TracerError::SceneLoad(format!(
                                "The object \"{}\" was already present in the scene.",
                                key
                            ))),
                            None => Ok(()),
                        }
                    }),
                GeometryData::Box { min, max, material } => materials
                    .get(&material)
                    .ok_or(TracerError::UnknownMaterial(material))
                    .and_then(|mat| {
                        match geometry.insert(key.clone(), create_box(Arc::clone(mat), min, max)) {
                            Some(_) => Err(TracerError::SceneLoad(format!(
                                "The object \"{}\" was already present in the scene.",
                                key
                            ))),
                            None => Ok(()),
                        }
                    }),
                GeometryData::RotateY { key, degrees } => {
                    rotations_y.insert(key.clone(), GeometryData::RotateY { key, degrees });
                    Ok(())
                }
                GeometryData::Translate { key, pos } => {
                    translations.insert(key.clone(), GeometryData::Translate { key, pos });
                    Ok(())
                }
            })?;

        // Rotations
        rotations_y
            .into_iter()
            .try_for_each::<_, Result<(), TracerError>>(|(child_key, rot)| match rot {
                GeometryData::RotateY { key, degrees } => geometry
                    .remove(child_key.as_str())
                    .ok_or_else(|| {
                        TracerError::SceneLoad(format!(
                            "Rotation_Y \"{}\" did not have any child with key \"{}\"",
                            key, child_key
                        ))
                    })
                    .map(|obj| {
                        geometry.insert(key, create_rotate_y(degrees, obj));
                    }),
                _ => Err(TracerError::SceneLoad(String::from(
                    "Expected rotations to be rotations",
                ))),
            })?;

        // Translations
        translations
            .into_iter()
            .try_for_each::<_, Result<(), TracerError>>(|(child_key, trans)| match trans {
                GeometryData::Translate { key, pos } => geometry
                    .remove(child_key.as_str())
                    .ok_or_else(|| {
                        TracerError::SceneLoad(format!(
                            "Translation \"{}\" did not have any child with key \"{}\"",
                            key, child_key
                        ))
                    })
                    .map(|obj| {
                        geometry.insert(key, create_translate(pos, obj));
                    }),
                _ => Err(TracerError::SceneLoad(String::from(
                    "Expected translations to be translations",
                ))),
            })?;

        Ok(SceneLoadData {
            objects: geometry.into_values().collect(),
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
