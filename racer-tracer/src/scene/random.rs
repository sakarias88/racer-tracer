use std::sync::Arc;

use crate::{
    background_color::Sky,
    config::CameraConfig,
    error::TracerError,
    geometry_creation::{create_movable_sphere, create_sphere},
    material::{dialectric::Dialectric, lambertian::Lambertian, metal::Metal},
    scene::SceneLoader,
    texture::{checkered::Checkered, solid_color::SolidColor},
    util::{random_double, random_double_range},
    vec3::{Color, Vec3},
};

use super::{SceneLoadData, SceneObject};

pub struct Random {}

impl Random {
    pub fn new() -> Self {
        Self {}
    }
}

impl SceneLoader for Random {
    fn load(&self) -> Result<SceneLoadData, TracerError> {
        let mut geometry: Vec<SceneObject> = Vec::new();
        let checkered = Arc::new(Checkered::new(
            Arc::new(SolidColor::new_from_rgb(0.2, 0.3, 0.1)),
            Arc::new(SolidColor::new_from_rgb(0.9, 0.9, 0.9)),
        ));
        let ground_material = Arc::new(Lambertian::new(checkered));
        geometry.push(create_sphere(
            ground_material,
            Vec3::new(0.0, -1000.0, 0.0),
            1000.0,
        ));

        for a in -11..11 {
            for b in -11..11 {
                let choose_mat = random_double();
                let center = Vec3::new(
                    a as f64 + 0.9 * random_double(),
                    0.2,
                    b as f64 + 0.9 * random_double(),
                );

                if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                    if choose_mat < 0.8 {
                        // diffuse
                        let albedo = Color::random() * Color::random();
                        let mat = Arc::new(Lambertian::new_with_color(albedo));
                        let center2 = center + Vec3::new(0.0, random_double_range(0.0, 0.5), 0.0);

                        geometry.push(create_movable_sphere(mat, center, center2, 0.2, 0.0, 1.0));
                    } else if choose_mat > 0.95 {
                        // metal
                        let albedo = Color::random_range(0.5, 1.0);
                        let fuzz = random_double_range(0.0, 0.5);
                        let mat = Arc::new(Metal::new_with_color(albedo, fuzz));
                        geometry.push(create_sphere(mat, center, 0.2));
                    } else {
                        // glass
                        let mat = Arc::new(Dialectric::new(1.5));
                        geometry.push(create_sphere(mat, center, 0.2));
                    }
                }
            }
        }

        let dial_mat = Arc::new(Dialectric::new(1.5));
        geometry.push(create_sphere(dial_mat, Vec3::new(0.0, 1.0, 0.0), 1.0));

        let lamb_mat = Arc::new(Lambertian::new_with_color(Color::new(0.4, 0.2, 0.1)));
        geometry.push(create_sphere(lamb_mat, Vec3::new(-4.0, 1.0, 0.0), 1.0));

        let metal_mat = Arc::new(Metal::new_with_color(Color::new(0.7, 0.6, 0.5), 0.0));

        geometry.push(create_sphere(metal_mat, Vec3::new(4.0, 1.0, 0.0), 1.0));

        Ok(SceneLoadData {
            objects: geometry,
            background: Box::<Sky>::default(),
            camera: Some(CameraConfig {
                vfov: Some(20.0),
                aperture: Some(0.1),
                focus_distance: Some(10.0),
                pos: Some(Vec3::new(0.0, 2.0, 10.0)),
                look_at: Some(Vec3::new(0.0, 0.0, 0.0)),
                speed: Some(0.000002),
                sensitivity: None,
            }),
            tone_map: None,
        })
    }
}
