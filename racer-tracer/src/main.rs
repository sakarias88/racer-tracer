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
    convert::TryFrom,
    sync::RwLock,
    time::{Duration, Instant},
    vec::Vec,
};

use minifb::{Key, Window, WindowOptions};
use synchronoise::SignalEvent;

use crate::{
    camera::Camera,
    config::{Args, Config},
    error::TracerError,
    render::render,
    scene::Scene,
};

fn run(config: Config) -> Result<(), TracerError> {
    let preview_render_data = config.preview;
    let render_data = config.render;
    let image = image::Image::new(config.screen.width, config.screen.height);
    let camera = RwLock::new(Camera::new(&image, 2.0, 1.0));
    let scene: Scene = config
        .scene
        .ok_or(TracerError::NoScene())
        .and_then(Scene::from_file)?;
    let screen_buffer: RwLock<Vec<u32>> = RwLock::new(vec![0; image.width * image.height]);

    let mut window_res: Result<(), TracerError> = Ok(());
    let move_camera = &camera;

    let render_image = SignalEvent::manual(false);
    let cancel_render = SignalEvent::manual(false);
    let exit = SignalEvent::manual(false);

    rayon::scope(|s| {
        s.spawn(|_| {
            loop {
                if exit.wait_timeout(Duration::from_secs(0)) {
                    return;
                }

                if render_image.wait_timeout(Duration::from_secs(0)) && render_image.status() {
                    let render_time = Instant::now();
                    render(
                        &screen_buffer,
                        &camera,
                        &image,
                        &scene,
                        &render_data,
                        Some(&cancel_render),
                        None,
                    );

                    println!(
                        "It took {} seconds to render the image.",
                        Instant::now().duration_since(render_time).as_millis()
                    );
                } else {
                    // Render preview
                    render(
                        &screen_buffer,
                        &camera,
                        &image,
                        &scene,
                        &preview_render_data,
                        None,
                        Some(preview_render_data.scale),
                    );
                }
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
                while window.is_open() && !window.is_key_down(Key::Escape) {
                    let dt = t.elapsed().as_micros() as f64 / 1000000.0;
                    t = Instant::now();
                    // Sleep a bit to not hog the lock on the buffer all the time.
                    std::thread::sleep(std::time::Duration::from_millis(10));

                    if window.is_key_released(Key::R) {
                        if render_image.status() {
                            cancel_render.signal();
                            render_image.reset();
                        } else {
                            render_image.signal();
                            cancel_render.reset();
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
                exit.signal();
                Ok(())
            });
        });
    });
    window_res
}
use structopt::StructOpt;
fn main() {
    if let Err(e) = Config::try_from(Args::from_args()).and_then(run) {
        eprintln!("{}", e);
        std::process::exit(e.into())
    }
}
