#[macro_use]
mod error;
mod camera;
mod geometry;
mod image;
mod ray;
mod scene;
mod util;
mod vec3;

use std::vec::Vec;

use futures::{select, stream::FuturesUnordered, stream::StreamExt};
use geometry::Hittable;
use minifb::{Key, Window, WindowOptions};

use crate::camera::Camera;
use crate::geometry::sphere::Sphere;
use crate::image::Image;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::util::random_double;
use crate::vec3::Vec3;

fn ray_color(scene: &Scene, ray: &Ray) -> Vec3 {
    if let Some(hit_record) = scene.hit(ray, 0.0, std::f64::INFINITY) {
        //return hit_record.color;
        return 0.5 * (hit_record.normal + Vec3::new(1.0, 1.0, 1.0));
    }

    // Sky
    let unit_direction = ray.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

// TODO: Rustify
async fn raytrace(
    scene: Scene,
    camera: Camera,
    image: Image,
    row: usize,
) -> Result<(usize, Vec<u32>), error::TracerError> {
    let mut buffer: Vec<u32> = vec![0; image.width as usize];
    let mut colors: Vec<Vec3> = vec![Vec3::default(); image.width as usize];

    for i in 0..buffer.len() {
        for _ in 0..image.samples_per_pixel {
            let u: f64 = (i as f64 + random_double()) / (image.width - 1) as f64;
            let v: f64 = (row as f64 + random_double()) / (image.height - 1) as f64;
            colors[i] += ray_color(&scene, &camera.get_ray(u, v));
        }
    }

    for i in 0..image.width {
        buffer[i] = (colors[i] / image.samples_per_pixel as f64).as_color();
    }

    Ok((row, buffer))
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

async fn run(
    rows_per_update: u32,
    aspect_ratio: f64,
    screen_width: usize,
    samples: usize,
) -> Result<(), error::TracerError> {
    let image = image::Image::new(aspect_ratio, screen_width, samples);
    let camera = camera::Camera::new(&image, 2.0, 1.0);
    let scene = create_scene();

    let mut screen_buffer: Vec<u32> = vec![0; image.width * image.height];
    let mut window = Window::new(
        "racer-tracer",
        image.width,
        image.height,
        WindowOptions::default(),
    )
    .expect("Unable to create window");
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut futs = FuturesUnordered::new();
    // One future per row is a bit high.
    // Could do something less spammy.
    for h in 0..image.height {
        // TODO: Either clone all or lock em all.
        futs.push(raytrace(scene.clone(), camera.clone(), image.clone(), h));
    }

    // TODO: use rayon
    // Since it's cooperative multitasking this is not really helpful at the moment.
    // You will get pretty much get the same result without the tokio asyncness.
    // using rayon with threads is a different matter.
    let mut complete = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !complete {
            for _ in 0..rows_per_update {
                select! {
                    res = futs.select_next_some() => {
                        let row_buffer = res.expect("Expected to get data");
                        let start = row_buffer.0 * image.width;
                        let end = start + image.width;
                        screen_buffer[start..end].copy_from_slice(row_buffer.1.as_slice());
                    },
                    complete => {
                        if !complete {
                            println!("Completed!");
                        }
                        complete = true;
                    },
                }
            }
        }

        window
            .update_with_buffer(&screen_buffer, image.width, image.height)
            .map_err(|e| error::TracerError::FailedToUpdateWindow(e.to_string()))?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run(100, 16.0 / 9.0, 1200, 100).await {
        eprintln!("{}", e);
        std::process::exit(e.into())
    }
}
