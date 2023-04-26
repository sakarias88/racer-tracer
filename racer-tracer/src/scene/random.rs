use std::sync::Arc;

use crate::{
    error::TracerError,
    material::{dialectric::Dialectric, lambertian::Lambertian, metal::Metal},
    scene::SceneLoader,
    util::{random_double, random_double_range},
    vec3::{Color, Vec3},
};

use super::SceneObject;

pub struct Random {}

impl Random {
    pub fn new() -> Self {
        Self {}
    }
}

impl SceneLoader for Random {
    fn load(&self) -> Result<Vec<SceneObject>, TracerError> {
        let mut geometry: Vec<SceneObject> = Vec::new();
        let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        geometry.push(crate::scene::create_sphere(
            Vec3::new(0.0, -1000.0, 0.0),
            ground_material,
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
                        let mat = Arc::new(Lambertian::new(albedo));
                        let center2 = center + Vec3::new(0.0, random_double_range(0.0, 0.5), 0.0);

                        geometry.push(crate::scene::create_movable_sphere(
                            center, center2, mat, 0.2, 0.0, 1.0,
                        ));
                    } else if choose_mat > 0.95 {
                        // metal
                        let albedo = Color::random_range(0.5, 1.0);
                        let fuzz = random_double_range(0.0, 0.5);
                        let mat = Arc::new(Metal::new(albedo, fuzz));
                        geometry.push(crate::scene::create_sphere(center, mat, 0.2));
                    } else {
                        // glass
                        let mat = Arc::new(Dialectric::new(1.5));
                        geometry.push(crate::scene::create_sphere(center, mat, 0.2));
                    }
                }
            }
        }

        let dial_mat = Arc::new(Dialectric::new(1.5));
        geometry.push(crate::scene::create_sphere(
            Vec3::new(0.0, 1.0, 0.0),
            dial_mat,
            1.0,
        ));

        let lamb_mat = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
        geometry.push(crate::scene::create_sphere(
            Vec3::new(-4.0, 1.0, 0.0),
            lamb_mat,
            1.0,
        ));

        let metal_mat = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

        geometry.push(crate::scene::create_sphere(
            Vec3::new(4.0, 1.0, 0.0),
            metal_mat,
            1.0,
        ));

        Ok(geometry)
    }
}
