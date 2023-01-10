#[macro_use]
mod error;
mod camera;
mod geometry;
mod image;
mod ray;
mod scene;
mod util;
mod vec3;

use std::{
    borrow::Borrow,
    sync::{Arc, Mutex, RwLock},
    vec::Vec,
};

use geometry::Hittable;
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;

use crate::camera::Camera;
use crate::error::TracerError;
use crate::geometry::sphere::Sphere;
use crate::image::Image;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::util::random_double;
use crate::vec3::Vec3;

fn ray_color(scene: &dyn Hittable, ray: &Ray) -> Vec3 {
    if let Some(hit_record) = scene.hit(ray, 0.0, std::f64::INFINITY) {
        //return hit_record.color;
        return 0.5 * (hit_record.normal + Vec3::new(1.0, 1.0, 1.0));
    }

    // TODO: make sky part of scene.
    // Sky
    let unit_direction = ray.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

fn raytrace(scene: &dyn Hittable, camera: &Camera, image: &Image, row: usize) -> Vec<u32> {
    let mut colors: Vec<Vec3> = vec![Vec3::default(); image.width as usize];
    colors.iter_mut().enumerate().for_each(|(i, col)| {
        for _ in 0..image.samples_per_pixel {
            let u: f64 = (i as f64 + random_double()) / (image.width - 1) as f64;
            let v: f64 = (row as f64 + random_double()) / (image.height - 1) as f64;
            *col += ray_color(scene, &camera.get_ray(u, v));
        }
    });

    // TODO: Could do rolling average
    let mut buffer: Vec<u32> = vec![0; image.width as usize];
    for i in 0..image.width {
        buffer[i] = (colors[i] / image.samples_per_pixel as f64).as_color();
    }

    buffer
}

type Data = (Arc<RwLock<Vec<u32>>>, Arc<Camera>, Arc<Image>, usize);

fn render(
    buffer: Arc<RwLock<Vec<u32>>>,
    camera: Arc<Camera>,
    image: Arc<Image>,
    scene: Box<dyn Hittable + std::marker::Sync>,
) {
    let scene: &(dyn Hittable + Sync) = scene.borrow();

    let v: Vec<Data> = (0..image.height)
        .map(|row| {
            (
                Arc::clone(&buffer),
                Arc::clone(&camera),
                Arc::clone(&image),
                row,
            )
        })
        .collect();

    v.par_iter().for_each(|(buf, camera, image, row)| {
        let start = row * image.width;
        let end = start + image.width;
        let col_buf = raytrace(scene.borrow(), camera, image, *row);
        let mut buf = buf.write().expect("Failed to get screen buffer lock."); // TODO: No except
        buf[start..end].copy_from_slice(col_buf.as_slice());
    });
}

fn create_scene() -> Scene {
    let mut scene = Scene::new();
    let sphere1 = Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, Vec3::new(0.0, 1.0, 0.0));
    let sphere2 = Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        Vec3::new(0.0, 1.0, 0.0),
    );
    scene.add(Box::new(sphere1));
    scene.add(Box::new(sphere2));
    scene
}

fn run(aspect_ratio: f64, screen_width: usize, samples: usize) -> Result<(), TracerError> {
    let image = Arc::new(image::Image::new(aspect_ratio, screen_width, samples));
    let camera = Arc::new(camera::Camera::new(&image, 2.0, 1.0));
    let scene: Box<dyn Hittable + Sync + Send> = Box::new(create_scene());
    let screen_buffer: Arc<RwLock<Vec<u32>>> =
        Arc::new(RwLock::new(vec![0; image.width * image.height]));

    let window_res: Arc<Mutex<Result<(), TracerError>>> = Arc::new(Mutex::new(Ok(())));

    rayon::scope(|s| {
        s.spawn(|_| {
            render(
                Arc::clone(&screen_buffer),
                camera,
                Arc::clone(&image),
                scene,
            )
        });
        s.spawn(|_| {
            let result = Window::new(
                "racer-tracer",
                image.width,
                image.height,
                WindowOptions::default(),
            )
            .map_err(|e| TracerError::FailedToCreateWindow(e.to_string()))
            .map(|mut window| {
                window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
                window
            })
            .and_then(|mut window| {
                // TODO: Only re-render window then buffer is changed
                while window.is_open() && !window.is_key_down(Key::Escape) {
                    // Sleep a bit to not hog the lock on the buffer all the time.
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    screen_buffer
                        .read()
                        .map_err(|e| TracerError::FailedToUpdateWindow(e.to_string()))
                        .and_then(|buf| {
                            window
                                .update_with_buffer(&buf, image.width, image.height)
                                .map_err(|e| TracerError::FailedToUpdateWindow(e.to_string()))
                        })?
                }
                Ok(())
            });

            if result.is_err() {
                let mut a = window_res.lock().expect("Failed to get result lock.");
                *a = result;
            }
        });
    });

    let res = (window_res.lock().expect("Failed to get result lock.")).clone();
    res
}

fn main() {
    if let Err(e) = run(16.0 / 9.0, 1200, 10) {
        eprintln!("{}", e);
        std::process::exit(e.into())
    }
}
