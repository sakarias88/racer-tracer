#[macro_use]
mod error;
mod camera;
mod config;
mod geometry;
mod image;
mod image_action;
mod key_inputs;
mod material;
mod ray;
mod renderer;
mod scene;
mod terminal;
mod util;
mod vec3;

extern crate image as img;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::{
    convert::TryFrom,
    fs::OpenOptions,
    path::Path,
    sync::RwLock,
    time::{Duration, Instant},
    vec::Vec,
};

use minifb::{Key, Window, WindowOptions};
use slog::{Drain, Logger};
use structopt::StructOpt;
use synchronoise::SignalEvent;
use terminal::Terminal;

use crate::{
    renderer::{RenderData, Renderer},
    vec3::Vec3,
};

use crate::{
    camera::Camera,
    config::{Args, Config},
    error::TracerError,
    image_action::ImageAction,
    key_inputs::KeyInputs,
    renderer::{cpu::CpuRenderer, cpu_scaled::CpuRendererScaled},
    scene::Scene,
};

fn run(config: Config, log: Logger, term: Terminal) -> Result<(), TracerError> {
    info!(log, "Starting racer-tracer {}", env!("CARGO_PKG_VERSION"));
    let renderer: &dyn Renderer = &CpuRenderer {} as &dyn Renderer;
    let renderer_preview: &dyn Renderer = &CpuRendererScaled {} as &dyn Renderer;
    let image = image::Image::new(config.screen.width, config.screen.height);
    let screen_buffer: RwLock<Vec<u32>> = RwLock::new(vec![0; image.width * image.height]);
    let look_from = Vec3::new(13.0, 2.0, 3.0);
    let look_at = Vec3::new(0.0, 0.0, 0.0);
    let camera_speed = 2.0;
    let camera = RwLock::new(Camera::new(
        look_from,
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        &image,
        0.1,
        10.0,
    ));

    let scene = Scene::try_new(&config.loader)?;

    let mut window_res: Result<(), TracerError> = Ok(());
    let mut render_res: Result<(), TracerError> = Ok(());

    let render_image = SignalEvent::manual(false);
    let exit = SignalEvent::manual(false);

    let image_action: Box<dyn ImageAction> = (&config.image_action).into();

    // Setting up controls
    let mut key_inputs = KeyInputs::new(log.new(o!("scope" => "key-intputs")));
    let render_image_fn = |_| {
        render_image.signal();
        Ok(())
    };
    key_inputs.release(Key::R, &render_image_fn);

    let go_forward = |dt: f64| {
        camera
            .write()
            .map_err(|e| TracerError::KeyError(e.to_string()))
            .map(|mut cam| {
                cam.go_forward(-dt * camera_speed);
            })
    };
    key_inputs.down(Key::W, &go_forward);

    let go_back = |dt: f64| {
        camera
            .write()
            .map_err(|e| TracerError::KeyError(e.to_string()))
            .map(|mut cam| {
                cam.go_forward(dt * camera_speed);
            })
    };
    key_inputs.down(Key::S, &go_back);

    let go_left = |dt: f64| {
        camera
            .write()
            .map_err(|e| TracerError::KeyError(e.to_string()))
            .map(|mut cam| {
                cam.go_right(-dt * camera_speed);
            })
    };
    key_inputs.down(Key::A, &go_left);

    let go_right = |dt: f64| {
        camera
            .write()
            .map_err(|e| TracerError::KeyError(e.to_string()))
            .map(|mut cam| {
                cam.go_right(dt * camera_speed);
            })
    };
    key_inputs.down(Key::D, &go_right);

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
                                    render_image.reset();
                                    // Render preview
                                    renderer_preview.render(RenderData {
                                        buffer: &screen_buffer,
                                        camera: &camera,
                                        image: &image,
                                        scene: &scene,
                                        config: &config,
                                        cancel_event: None,
                                    })
                                },
                                |_| {
                                    render_image.reset();
                                    let render_time = Instant::now();
                                    renderer
                                        .render(RenderData {
                                            buffer: &screen_buffer,
                                            camera: &camera,
                                            image: &image,
                                            scene: &scene,
                                            config: &config,
                                            cancel_event: Some(&render_image),
                                        })
                                        .and_then(|_| {
                                            info!(
                                                log,
                                                "It took {} seconds to render the image.",
                                                Instant::now()
                                                    .duration_since(render_time)
                                                    .as_secs()
                                            );
                                            image_action.action(
                                                &screen_buffer,
                                                &render_image,
                                                &config,
                                                log.new(o!("scope" => "image-action")),
                                                &term,
                                            )
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
                    key_inputs.update(&window, dt);
                    // Sleep a bit to not hog the lock on the buffer all the time.
                    std::thread::sleep(std::time::Duration::from_millis(1));

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
                render_image.signal();
                Ok(())
            });
        });
    });

    window_res.and(render_res)
}

fn create_log(log_file: &Path) -> Result<Logger, TracerError> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_file)
        .map(slog_term::PlainDecorator::new)
        .map(|log| slog_term::FullFormat::new(log).build().fuse())
        .map_err(|e| TracerError::CreateLogError(e.to_string()))
        .map(|file_drain| {
            let term_drain = slog_term::FullFormat::new(slog_term::TermDecorator::new().build())
                .build()
                .fuse();
            (file_drain, term_drain)
        })
        .map(|(file_drain, term_drain)| {
            let combined =
                slog_async::Async::new(slog::Duplicate::new(term_drain, file_drain).fuse())
                    .build()
                    .fuse();
            Logger::root(combined, o!())
        })
}

fn main() {
    let log_file = std::env::temp_dir().join("racer-tracer.log");
    let log = create_log(log_file.as_ref()).expect("Expected to be able to create a log");
    let term = Terminal::new(log.new(o!("scope" => "terminal")));
    terminal::write_term!(term, &format!("Log file: {}", log_file.display()));

    match Config::try_from(Args::from_args())
        .and_then(|config| run(config, log.new(o!("scope" => "run")), term))
    {
        Err(TracerError::ExitEvent) => {}
        Ok(_) => {}
        Err(e) => {
            error!(log, "{}", e);
            let exit_code = i32::from(e);
            error!(log, "Exiting with: {}", exit_code);
            std::process::exit(exit_code)
        }
    }
}
