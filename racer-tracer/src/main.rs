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

extern crate image as img;

use std::{
    convert::TryFrom,
    path::PathBuf,
    sync::RwLock,
    time::{Duration, Instant},
    vec::Vec,
};

use minifb::{Key, Window, WindowOptions};
use sha2::{Digest, Sha256};
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
    let camera = RwLock::new(Camera::new(
        Vec3::new(-2.0, 2.0, 1.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        90.0,
        &image,
        1.0,
    ));
    let scene: Scene = config
        .scene
        .ok_or(TracerError::NoScene())
        .and_then(Scene::from_file)?;

    let mut window_res: Result<(), TracerError> = Ok(());
    let mut render_res: Result<(), TracerError> = Ok(());

    let render_image = SignalEvent::manual(false);
    let cancel_render = SignalEvent::manual(false);
    let exit = SignalEvent::manual(false);

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
                                        screen_buffer
                                            .read()
                                            .map_err(|e| {
                                                TracerError::FailedToAcquireLock(e.to_string())
                                            })
                                            .map(|buf| {
                                                // Convert ARGB8 to RGBA8
                                                buf.iter()
                                                    .map(|v| {
                                                        let a: u32 = (v >> 24) & 0xff;
                                                        let r: u32 = (v >> 16) & 0xff;
                                                        let g: u32 = (v >> 8) & 0xff;
                                                        let b: u32 = v & 0xff;

                                                        (r << 24) | (g << 16) | (b << 8) | a
                                                    })
                                                    .flat_map(|val| val.to_be_bytes())
                                                    .collect::<Vec<u8>>()
                                            })
                                    })
                                    .and_then(|buf| {
                                        match &config.image_output_dir {
                                            Some(image_dir) => {
                                                println!("Saving image...");
                                                let mut sha = Sha256::new();

                                                sha.update(buf.as_slice());

                                                let mut file_path = PathBuf::from(image_dir);
                                                file_path.push(format!("{:X}.png", sha.finalize()));

                                                img::save_buffer(
                                                    file_path.as_path(),
                                                    buf.as_slice(),
                                                    config.screen.width as u32,
                                                    config.screen.height as u32,
                                                    img::ColorType::Rgba8,
                                                )
                                                .map_err(|e| {
                                                    let error = e.to_string();
                                                    TracerError::ImageSave(error)
                                                })
                                                .map(|_| {
                                                    println!(
                                                        "Saved image to: {}",
                                                        file_path.to_string_lossy()
                                                    );
                                                })
                                            }
                                            None => Ok(()),
                                        }
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
