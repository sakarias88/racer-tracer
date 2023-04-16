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
mod scene_controller;
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
    time::{Duration, Instant},
};

use minifb::{Key, Window, WindowOptions};
use slog::{Drain, Logger};
use structopt::StructOpt;
use synchronoise::SignalEvent;
use terminal::Terminal;

use crate::scene_controller::{interactive::InteractiveScene, SceneController, SceneData};

use crate::{
    camera::Camera,
    config::{Args, Config},
    error::TracerError,
    key_inputs::KeyInputs,
    scene::Scene,
};

fn run(config: Config, log: Logger, term: Terminal) -> Result<(), TracerError> {
    info!(log, "Starting racer-tracer {}", env!("CARGO_PKG_VERSION"));
    let image = image::Image::new(config.screen.width, config.screen.height);
    let camera = Camera::from((&image, &config.camera));
    let scene = Scene::try_new(&config.loader)?;
    let mut window_res: Result<(), TracerError> = Ok(());
    let mut render_res: Result<(), TracerError> = Ok(());
    let exit = SignalEvent::manual(false);

    let scene_controller = {
        match &config.scene_controller {
            config::ConfigSceneController::Interactive => {
                let camera_speed = 0.000002;
                let camera_sensitivity = 0.001;
                InteractiveScene::new(
                    SceneData {
                        log: log.new(o!("scope" => "scene-controller")),
                        term,
                        config: config.clone(),
                        scene,
                        camera,
                        image: image.clone(),
                    },
                    camera_speed,
                    camera_sensitivity,
                )
            }
        }
    };

    let mut inputs = KeyInputs::new(log.new(o!("scope" => "key-inputs")));
    inputs.register_inputs(scene_controller.key_inputs());
    if let Some(mouse_cb) = scene_controller.mouse_input() {
        inputs.mouse_move(mouse_cb);
    }

    rayon::scope(|s| {
        // Render
        s.spawn(|_| {
            while render_res.is_ok() {
                render_res = (!exit.wait_timeout(Duration::from_secs(0)))
                    .then_some(|| ())
                    .ok_or(TracerError::ExitEvent)
                    .and_then(|_| scene_controller.render());
            }

            if render_res.is_err() {
                exit.signal();
            }
        });

        // Update
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
            .map(|mut window| {
                let mut t = Instant::now();
                while window_res.is_ok()
                    && window.is_open()
                    && !window.is_key_down(Key::Escape)
                    && !exit.status()
                {
                    let dt = t.elapsed().as_micros() as f64;
                    t = Instant::now();
                    inputs.update(&mut window, dt);

                    window_res =
                        scene_controller
                            .get_buffer()
                            .and_then(|maybe_buf| match maybe_buf {
                                Some(buf) => window
                                    .update_with_buffer(&buf, image.width, image.height)
                                    .map_err(|e| TracerError::FailedToUpdateWindow(e.to_string())),
                                None => {
                                    window.update();
                                    Ok(())
                                }
                            });
                }
                exit.signal();
                scene_controller.stop()
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
            error!(log, "Error: {}", e);
            let exit_code = i32::from(e);
            error!(log, "Exiting with: {}", exit_code);
            std::process::exit(exit_code)
        }
    }
}
