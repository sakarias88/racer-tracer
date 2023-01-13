#[macro_use]
mod error;
mod camera;
mod geometry;
mod image;
mod material;
mod ray;
mod render;
mod scene;
mod util;
mod vec3;

use std::{
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
    vec::Vec,
};

use material::{lambertian::Lambertian, metal::Metal, Material};
use minifb::{Key, Window, WindowOptions};
use synchronoise::SignalEvent;

use crate::{
    camera::Camera,
    error::TracerError,
    geometry::sphere::Sphere,
    geometry::Hittable,
    image::SubImage,
    render::render,
    scene::Scene,
    vec3::{Color, Vec3},
};

type SharedMaterial = Arc<Box<dyn Material>>;

// TODO: Read from yml
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
}

fn run(
    aspect_ratio: f64,
    screen_width: usize,
    samples: usize,
    max_depth: usize,
    recurse_depth: usize,
) -> Result<(), TracerError> {
    let image = image::Image::new(aspect_ratio, screen_width, samples);
    let camera = Arc::new(RwLock::new(Camera::new(&image, 2.0, 1.0)));
    let scene: Arc<Box<dyn Hittable>> = Arc::new(Box::new(create_scene()));
    let screen_buffer: Arc<RwLock<Vec<u32>>> =
        Arc::new(RwLock::new(vec![0; image.width * image.height]));

    let window_res: Arc<Mutex<Result<(), TracerError>>> = Arc::new(Mutex::new(Ok(())));
    let sub_image: SubImage = (&image).into();
    let move_camera = Arc::clone(&camera);

    let render_image = Arc::new(SignalEvent::manual(false));
    let window_render_image = Arc::clone(&render_image);

    let cancel_render = Arc::new(SignalEvent::manual(false));
    let window_cancel_render = cancel_render.clone();

    let exit = Arc::new(SignalEvent::manual(false));
    let window_exit = Arc::clone(&exit);

    rayon::scope(|s| {
        s.spawn(|_| {
            // TODO: Make configurable
            let preview_scale = 4;
            let preview_samples = 2;
            let preview_max_depth = 4;
            let preview_recurse_depth = 4;

            loop {
                if exit.wait_timeout(Duration::from_secs(0)) {
                    return;
                }

                if render_image.wait_timeout(Duration::from_secs(0)) && render_image.status() {
                    let render_time = Instant::now();
                    let cancel_render_event = Arc::clone(&cancel_render);
                    render(
                        Arc::clone(&screen_buffer),
                        Arc::clone(&camera),
                        &sub_image,
                        Arc::clone(&scene),
                        samples,
                        1,
                        max_depth,
                        recurse_depth,
                        Some(cancel_render_event),
                    );

                    println!(
                        "It took {} seconds to render the image.",
                        Instant::now().duration_since(render_time).as_millis()
                    );
                } else {
                    // Render preview
                    render(
                        Arc::clone(&screen_buffer),
                        Arc::clone(&camera),
                        &sub_image,
                        Arc::clone(&scene),
                        preview_samples,
                        preview_scale,
                        preview_max_depth,
                        // TODO: Could create a function to create the optimal value
                        preview_recurse_depth, //recursive thread depth
                        None,
                    );
                }
            }
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
                let mut t = Instant::now();
                while window.is_open() && !window.is_key_down(Key::Escape) {
                    let dt = t.elapsed().as_micros() as f64 / 1000000.0;
                    t = Instant::now();
                    // Sleep a bit to not hog the lock on the buffer all the time.
                    std::thread::sleep(std::time::Duration::from_millis(10));

                    if window.is_key_released(Key::R) {
                        if window_render_image.status() {
                            window_cancel_render.signal();
                            window_render_image.reset();
                        } else {
                            window_render_image.signal();
                            window_cancel_render.reset();
                        }
                    }

                    {
                        let mut cam = move_camera.write().expect("TODO");
                        if window.is_key_down(Key::W) {
                            cam.go_forward(-dt);
                        } else if window.is_key_down(Key::S) {
                            cam.go_forward(dt);
                        }

                        if window.is_key_down(Key::A) {
                            cam.go_right(-dt);
                        } else if window.is_key_down(Key::D) {
                            cam.go_right(dt);
                        }
                    }

                    screen_buffer
                        .read()
                        .map_err(|e| TracerError::FailedToUpdateWindow(e.to_string()))
                        .and_then(|buf| {
                            window
                                .update_with_buffer(&buf, image.width, image.height)
                                .map_err(|e| TracerError::FailedToUpdateWindow(e.to_string()))
                        })?
                }
                window_exit.signal();
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
    // TODO: Read configuration and args
    let samples = 1000; // Samples per pixel
    let max_depth = 50; // Max ray trace depth
    let recurse_depth = 4; // How many times the screen with split itself into sub images each time splitting it into 4 new smaller ones.
    if let Err(e) = run(16.0 / 9.0, 1280, samples, max_depth, recurse_depth) {
        eprintln!("{}", e);
        std::process::exit(e.into())
    }
}
