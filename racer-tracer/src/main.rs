#[macro_use]
mod error;
mod camera;
mod geometry;
mod image;
mod material;
mod ray;
mod scene;
mod util;
mod vec3;

use std::{
    borrow::Borrow,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
    vec::Vec,
};

use material::{lambertian::Lambertian, metal::Metal, Material};
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use vec3::Color;

use crate::{
    camera::Camera,
    error::TracerError,
    geometry::sphere::Sphere,
    geometry::Hittable,
    image::Image,
    ray::Ray,
    scene::Scene,
    util::random_double,
    vec3::Vec3,
    vec3::{random_in_hemisphere, random_unit_vector},
};

fn ray_color(scene: &dyn Hittable, ray: &Ray, depth: usize) -> Vec3 {
    if depth == 0 {
        return Vec3::default();
    }

    if let Some(rec) = scene.hit(ray, 0.001, std::f64::INFINITY) {
        if let Some((scattered, attenuation)) = rec.material.scatter(ray, &rec) {
            return attenuation * ray_color(scene, &scattered, depth - 1);
        }
        return Color::default();
        //let target = rec.point + random_in_hemisphere(&rec.normal);
        //let target = rec.point + rec.normal + random_unit_vector();
        //return 0.5 * ray_color(scene, &Ray::new(rec.point, target - rec.point), depth - 1);
        //return hit_record.color;
        //return 0.5 * (hit_record.normal + Vec3::new(1.0, 1.0, 1.0));
    }

    // TODO: make sky part of scene.
    // Sky
    let unit_direction = ray.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

fn flush(
    row: usize,
    width: usize,
    start: usize,
    count: usize,
    samples: usize,
    source: &[Vec3],
    dest: &RwLock<Vec<u32>>,
) {
    let mut buf = dest
        .write()
        .expect("Failed to get write guard when flushing data.");

    for i in 0..count {
        buf[row * width + start + i] = source[start + i].scale_sqrt(samples).as_color();
    }
}

fn raytrace(
    buffer: &RwLock<Vec<u32>>,
    scene: &dyn Hittable,
    camera: &Camera,
    image: &Image,
    row: usize,
    max_depth: usize,
) {
    let mut now = Instant::now();
    let flush_delay = Duration::from_millis(50 + ((random_double() * 2000.0) as u64));
    let mut flush_start: usize = 0;
    let mut colors: Vec<Vec3> = vec![Vec3::default(); image.width as usize];

    for i in 0..colors.len() {
        for _ in 0..image.samples_per_pixel {
            let u: f64 = (i as f64 + random_double()) / (image.width - 1) as f64;
            let v: f64 = (row as f64 + random_double()) / (image.height - 1) as f64;
            colors[i] += ray_color(scene, &camera.get_ray(u, v), max_depth);
        }

        // Update the screen buffer every now and again.
        if now.elapsed() > flush_delay {
            now = Instant::now();
            flush(
                row,
                image.width,
                flush_start,
                i - flush_start,
                image.samples_per_pixel,
                colors.as_slice(),
                buffer,
            );
            flush_start = i;
        }
    }

    // Flush last part of the buffer.
    flush(
        row,
        image.width,
        flush_start,
        colors.len() - flush_start,
        image.samples_per_pixel,
        colors.as_slice(),
        buffer,
    );
}

type Data = (Arc<RwLock<Vec<u32>>>, Arc<Camera>, Arc<Image>, usize);

fn render(
    buffer: Arc<RwLock<Vec<u32>>>,
    camera: Arc<Camera>,
    image: Arc<Image>,
    scene: Box<dyn Hittable + std::marker::Sync>,
    max_depth: usize,
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
        raytrace(buf, scene, camera, image, *row, max_depth);
    });
}

type SharedMaterial = Arc<Box<dyn Material + Send + Sync>>;
fn create_scene() -> Scene {
    let mut scene = Scene::new();
    let material_ground: SharedMaterial =
        Arc::new(Box::new(Lambertian::new(Color::new(0.8, 0.8, 0.0))));
    let material_center: SharedMaterial =
        Arc::new(Box::new(Lambertian::new(Color::new(0.7, 0.3, 0.3))));
    let material_left: SharedMaterial =
        Arc::new(Box::new(Metal::new(Color::new(0.8, 0.8, 0.8), 0.3)));
    let material_right: SharedMaterial =
        Arc::new(Box::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.1)));

    scene.add(Box::new(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        Arc::clone(&material_ground),
    )));
    scene.add(Box::new(Sphere::new(
        Vec3::new(0.0, 0.0, -1.0),
        0.5,
        Arc::clone(&material_center),
    )));
    scene.add(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        Arc::clone(&material_left),
    )));
    scene.add(Box::new(Sphere::new(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        Arc::clone(&material_right),
    )));
    scene
    // Materials
    /*    let red: Arc<Box<dyn Material + Send + Sync>> =
        Arc::new(Box::new(Lambertian::new(Color::new(1.0, 0.0, 0.0))));
    let green: Arc<Box<dyn Material + Send + Sync>> =
        Arc::new(Box::new(Lambertian::new(Color::new(0.0, 1.0, 0.0))));
    let metal_red: Arc<Box<dyn Material + Send + Sync>> =
        Arc::new(Box::new(Metal::new(Color::new(1.0, 0.0, 0.0))));

    // Geometry
    let sphere1 = Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, Arc::clone(&metal_red));
    let sphere2 = Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0, Arc::clone(&green));
    scene.add(Box::new(sphere1));
    scene.add(Box::new(sphere2));
    scene*/
}

fn run(
    aspect_ratio: f64,
    screen_width: usize,
    samples: usize,
    max_depth: usize,
) -> Result<(), TracerError> {
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
                max_depth,
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
                while window.is_open() && !window.is_key_down(Key::Escape) {
                    // Sleep a bit to not hog the lock on the buffer all the time.
                    std::thread::sleep(std::time::Duration::from_millis(100));

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
    if let Err(e) = run(16.0 / 9.0, 1200, 1000, 50) {
        eprintln!("{}", e);
        std::process::exit(e.into())
    }
}
