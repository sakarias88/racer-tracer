#[macro_use]
mod error;
mod camera;
mod config;
mod geometry;
mod image;
mod image_action;
mod material;
mod ray;
mod render;
mod scene;
mod util;
mod vec3;

extern crate image as img;

use std::{
    convert::TryFrom,
    sync::RwLock,
    time::{Duration, Instant},
    vec::Vec,
};

use image_action::ImageAction;
use minifb::{Key, Window, WindowOptions};
use synchronoise::SignalEvent;

use crate::vec3::Vec3;

use crate::{
    camera::Camera,
    config::{Args, Config},
    error::TracerError,
    render::render,
    scene::Scene,
};

fn run(config: Config) -> Result<(), TracerError> {
    let image = image::Image::new(config.screen.width, config.screen.height);
    let screen_buffer: RwLock<Vec<u32>> = RwLock::new(vec![0; image.width * image.height]);
    let look_from = Vec3::new(13.0, 2.0, 3.0);
    let look_at = Vec3::new(0.0, 0.0, 0.0);
    let camera = RwLock::new(Camera::new(
        look_from,
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        &image,
        0.1,
        10.0,
    ));

    let scene = Scene::try_new((&config.loader).into())?;

    let mut window_res: Result<(), TracerError> = Ok(());
    let mut render_res: Result<(), TracerError> = Ok(());

    let render_image = SignalEvent::manual(false);
    let cancel_render = SignalEvent::manual(false);
    let exit = SignalEvent::manual(false);

    let image_action: Box<dyn ImageAction> = (&config.image_action).into();

    rayon::scope(|s| {
        s.spawn(|_| {
            while render_res.is_ok() {
                render_res = (!exit.wait_timeout(Duration::from_secs(0)))
                    .then_some(|| ())
                    .ok_or(TracerError::ExitEvent)
                    .and_then(|_| {
                        render_image
                            .wait_timeout(Duration::from_secs(0))
                            .then_some(|| ())
                            .map_or_else(
                                || {
                                    // Render preview
                                    render(
                                        &screen_buffer,
                                        &camera,
                                        &image,
                                        &scene,
                                        &config.preview,
                                        None,
                                        Some(config.preview.scale),
                                    )
                                },
                                |_| {
                                    let render_time = Instant::now();
                                    render(
                                        &screen_buffer,
                                        &camera,
                                        &image,
                                        &scene,
                                        &config.render,
                                        Some(&cancel_render),
                                        None,
                                    )
                                    .and_then(|_| {
                                        render_image.reset();
                                        println!(
                                            "It took {} seconds to render the image.",
                                            Instant::now().duration_since(render_time).as_secs()
                                        );
                                        image_action.action(&screen_buffer, &render_image, &config)
                                    })
                                },
                            )
                    });
            }

            if render_res.is_err() {
                exit.signal();
            }
        });

        s.spawn(|_| {
            window_res = Window::new(
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
                while window.is_open() && !window.is_key_down(Key::Escape) && !exit.status() {
                    let dt = t.elapsed().as_micros() as f64 / 1000000.0;
                    t = Instant::now();
                    // Sleep a bit to not hog the lock on the buffer all the time.
                    std::thread::sleep(std::time::Duration::from_millis(10));

                    if window.is_key_released(Key::R) {
                        if render_image.status() {
                            // Signal cancel
                            cancel_render.signal();
                            render_image.reset();
                        } else {
                            // Signal render
                            render_image.signal();
                            cancel_render.reset();
                        }
                    }

                    camera
                        .write()
                        .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
                        .map(|mut cam| {
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
                        })?;

                    screen_buffer
                        .read()
                        .map_err(|e| TracerError::FailedToUpdateWindow(e.to_string()))
                        .and_then(|buf| {
                            window
                                .update_with_buffer(&buf, image.width, image.height)
                                .map_err(|e| TracerError::FailedToUpdateWindow(e.to_string()))
                        })?
                }
                exit.signal();
                cancel_render.signal();
                Ok(())
            });
        });
    });

    window_res.and(render_res)
}
use structopt::StructOpt;
fn main() {
    match Config::try_from(Args::from_args()).and_then(run) {
        Err(TracerError::ExitEvent) => {}
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(e.into())
        }
    }
}
