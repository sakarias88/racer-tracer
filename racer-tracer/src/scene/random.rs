use std::sync::Arc;

use crate::{
    error::TracerError,
    geometry::{sphere::Sphere, Hittable},
    material::{dialectric::Dialectric, lambertian::Lambertian, metal::Metal, SharedMaterial},
    scene::SceneLoader,
    util::{random_double, random_double_range},
    vec3::{Color, Vec3},
};

pub struct Random {}

impl Random {
    pub fn new() -> Self {
        Self {}
    }
}

impl SceneLoader for Random {
    fn load(&self) -> Result<Vec<Box<dyn crate::geometry::Hittable>>, TracerError> {
        let mut geometry: Vec<Box<dyn Hittable>> = Vec::new();
        let ground_material: SharedMaterial =
            Arc::new(Box::new(Lambertian::new(Color::new(0.5, 0.5, 0.5))));
        geometry.push(Box::new(Sphere::new(
            Vec3::new(0.0, -1000.0, 0.0),
            1000.0,
            ground_material,
        )));

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
                        let mat: SharedMaterial = Arc::new(Box::new(Lambertian::new(albedo)));
                        geometry.push(Box::new(Sphere::new(center, 0.2, mat)));
                    } else if choose_mat > 0.95 {
                        // metal
                        let albedo = Color::random_range(0.5, 1.0);
                        let fuzz = random_double_range(0.0, 0.5);
                        let mat: SharedMaterial = Arc::new(Box::new(Metal::new(albedo, fuzz)));
                        geometry.push(Box::new(Sphere::new(center, 0.2, mat)));
                    } else {
                        // glass
                        let mat: SharedMaterial = Arc::new(Box::new(Dialectric::new(1.5)));
                        geometry.push(Box::new(Sphere::new(center, 0.2, mat)));
                    }
                }
            }
        }

        let dial_mat: SharedMaterial = Arc::new(Box::new(Dialectric::new(1.5)));
        geometry.push(Box::new(Sphere::new(
            Vec3::new(0.0, 1.0, 0.0),
            1.0,
            dial_mat,
        )));

        let lamb_mat: SharedMaterial =
            Arc::new(Box::new(Lambertian::new(Color::new(0.4, 0.2, 0.1))));
        geometry.push(Box::new(Sphere::new(
            Vec3::new(-4.0, 1.0, 0.0),
            1.0,
            lamb_mat,
        )));

        let metal_mat: SharedMaterial =
            Arc::new(Box::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0)));

        geometry.push(Box::new(Sphere::new(
            Vec3::new(4.0, 1.0, 0.0),
            1.0,
            metal_mat,
        )));

        Ok(geometry)
    }
}
