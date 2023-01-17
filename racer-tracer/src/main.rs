#[macro_use]
mod error;
mod camera;
mod config;
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

use material::{dialectric::Dialectric, lambertian::Lambertian, metal::Metal, Material};
use minifb::{Key, Window, WindowOptions};
use synchronoise::SignalEvent;

use crate::{
    camera::Camera,
    config::{Args, Config},
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
        Arc::new(Box::new(Lambertian::new(Color::new(0.1, 0.2, 0.5))));
    let material_left: SharedMaterial = Arc::new(Box::new(Dialectric::new(1.5)));

    let material_right: SharedMaterial =
        Arc::new(Box::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0)));

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
        Vec3::new(-1.0, 0.0, -1.0),
        -0.4,
        Arc::clone(&material_left),
    )));

    scene.add(Box::new(Sphere::new(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        Arc::clone(&material_right),
    )));
    scene
}

fn run(config: Config) -> Result<(), TracerError> {
    let preview_render_data = Arc::new(config.preview);
    let recurse_depth = 4;
    let render_data = Arc::new(config.render);
    let image = image::Image::new(config.screen.width, config.screen.height);
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
                        Arc::clone(&render_data),
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
                        Arc::clone(&preview_render_data),
                        recurse_depth,
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
use structopt::StructOpt;
fn main() {
    let args = Args::from_args();

    if let Err(e) = Config::from_file(args.config).and_then(run) {
        eprintln!("{}", e);
        std::process::exit(e.into())
    }
}
