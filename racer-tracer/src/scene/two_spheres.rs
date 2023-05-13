use std::{path::PathBuf, sync::Arc};

use crate::{
    error::TracerError,
    material::{dialectric::Dialectric, lambertian::Lambertian, metal::Metal, Material},
    texture::{checkered::Checkered, image::TextureImage, noise::Noise, solid_color::SolidColor},
    vec3::{Color, Vec3},
};

use super::{SceneLoader, SceneObject};

pub struct TwoSpheres {}

impl SceneLoader for TwoSpheres {
    fn load(&self) -> Result<Vec<SceneObject>, TracerError> {
        let mut geometry: Vec<SceneObject> = Vec::new();
        let perlin = Arc::new(Noise::new(4.0, Some(7), Color::new(1.0, 1.0, 1.0)));
        let checkered_texture = Arc::new(Checkered::new(
            Arc::new(SolidColor::new_from_rgb(0.5, 1.0, 0.5)),
            Arc::new(SolidColor::new_from_rgb(0.8, 0.8, 0.8)),
        ));

        let earth = Arc::new(TextureImage::try_new(&PathBuf::from(
            "../resources/images/earthmap.jpg",
        ))?);
        let earth_material: Arc<dyn Material> = Arc::new(Lambertian::new(earth));
        let obama = Arc::new(TextureImage::try_new(&PathBuf::from(
            "../resources/images/someone.png",
        ))?);
        let obama_material: Arc<dyn Material> = Arc::new(Metal::new(obama, 0.0));

        let ball_material: Arc<dyn Material> = Arc::new(Lambertian::new(perlin));
        let ground_material: Arc<dyn Material> = Arc::new(Lambertian::new(checkered_texture));
        let dialectric_material = Arc::new(Dialectric::new(1.5));
        geometry.push(crate::scene::create_sphere(
            Vec3::new(0.0, -1000.0, 0.0),
            Arc::clone(&ground_material),
            1000.0,
        ));
        geometry.push(crate::scene::create_sphere(
            Vec3::new(0.0, 1.0, 0.0),
            ball_material,
            1.0,
        ));
        geometry.push(crate::scene::create_sphere(
            Vec3::new(0.0, 1.0, 2.0),
            earth_material,
            1.0,
        ));
        geometry.push(crate::scene::create_sphere(
            Vec3::new(0.0, 1.0, -2.0),
            obama_material,
            1.0,
        ));
        geometry.push(crate::scene::create_sphere(
            Vec3::new(0.0, 1.0, -4.0),
            dialectric_material,
            1.0,
        ));

        Ok(geometry)
    }
}
