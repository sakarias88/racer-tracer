#[macro_use]
mod error;
mod camera;
mod image;
mod ray;
mod util;
mod vec3;

use crate::vec3::Vec3;
use std::vec::Vec;

use futures::{select, stream::FuturesUnordered, stream::StreamExt};
use minifb::{Key, Window, WindowOptions};

fn ray_color(ray: &ray::Ray) -> Vec3 {
    let unit_direction = vec3::unit_vector(ray.direction());
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

async fn raytrace(
    camera: camera::Camera,
    image: image::Image,
    row: usize,
) -> Result<(usize, Vec<u32>), error::TracerError> {
    let mut buffer: Vec<u32> = vec![0; image.width as usize];

    for i in 0..buffer.len() {
        let u: f64 = i as f64 / (image.width - 1) as f64;
        let v: f64 = row as f64 / (image.height - 1) as f64;
        let ray = ray::Ray::new(
            camera.origin,
            camera.lower_left_corner + u * camera.horizontal + v * camera.vertical - camera.origin,
        );
        let col = ray_color(&ray);
        buffer[i] = col.as_color();
    }

    Ok((row, buffer))
}

async fn run(
    rows_per_update: u32,
    aspect_ratio: f64,
    screen_height: usize,
) -> Result<(), error::TracerError> {
    let image = image::Image::new(aspect_ratio, screen_height);
    let camera = camera::Camera::new(&image, 2.0, 1.0);

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
        futs.push(raytrace(camera.clone(), image.clone(), h));
    }

    let mut complete = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !complete {
            for _ in 1..rows_per_update {
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
    if let Err(e) = run(50, 16.0 / 9.0, 1200).await {
        eprintln!("{}", e);
        std::process::exit(e.into())
    }
}
