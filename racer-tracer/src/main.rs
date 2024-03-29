#[macro_use]
mod error;
mod aabb;
mod background_color;
mod bvh_node;
mod camera;
mod config;
mod data_bus;
mod geometry;
mod geometry_creation;
mod image;
mod image_action;
mod image_buffer;
mod key_inputs;
mod material;
mod ray;
mod renderer;
mod scene;
mod scene_controller;
mod shared_scene;
mod terminal;
mod texture;
mod tone_map;
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
    sync::Arc,
    time::{Duration, Instant},
};

use minifb::{Key, Window, WindowOptions};
use slog::{Drain, Logger};
use structopt::StructOpt;
use synchronoise::SignalEvent;
use terminal::Terminal;

use crate::{
    background_color::BackgroundColor,
    bvh_node::BoundingVolumeHirearchy,
    camera::{CameraData, CameraInitData},
    config::SceneLoaderConfig as CLoader,
    image_action::ImageAction,
    image_buffer::{ImageBuffer, ScreenBuffer},
    renderer::Renderer,
    scene::{
        none::NoneLoader, random::Random, sandbox::Sandbox, yml::YmlLoader, Scene, SceneLoader,
    },
    scene_controller::{interactive::InteractiveScene, SceneController},
    tone_map::ToneMap,
    vec3::Vec3,
};

use crate::{
    camera::Camera,
    config::{Args, Config},
    error::TracerError,
    key_inputs::KeyInputs,
};

fn run(config: Config, log: Logger, _term: Terminal) -> Result<(), TracerError> {
    info!(log, "Starting racer-tracer {}", env!("CARGO_PKG_VERSION"));
    let image = image::Image::new(config.screen.width, config.screen.height);
    let loader = match &config.loader {
        CLoader::Yml { path } => Box::new(YmlLoader::new(path.clone())) as Box<dyn SceneLoader>,
        CLoader::Random => Box::new(Random::new()) as Box<dyn SceneLoader>,
        CLoader::None => Box::new(NoneLoader::new()) as Box<dyn SceneLoader>,
        CLoader::Sandbox => Box::new(Sandbox::new()) as Box<dyn SceneLoader>,
    };

    let scene_data = loader.load()?;
    let background = &*scene_data.background as &dyn BackgroundColor;
    let image_action: &dyn ImageAction = (&config.image_action).into();
    let tone_map: Box<dyn ToneMap> = scene_data
        .tone_map
        .unwrap_or_else(|| (&config.tone_map).into());

    let mut screen_data_buffer = vec![0; image.width * image.height];
    let mut image_buffer = ImageBuffer::new(image.clone());
    let mut screen_buffer =
        ScreenBuffer::new(image.clone(), image_buffer.get_data_writer(), tone_map);
    let screen_buffer_writer = screen_buffer.get_writer();
    let mut image_buffer_reader = image_buffer.get_reader();

    let camera_data =
        CameraData::merge(scene_data.camera.unwrap_or_default(), config.camera.clone());
    let mut camera = Camera::new(
        CameraInitData {
            look_from: camera_data.pos,
            look_at: camera_data.look_at,
            scene_up: Vec3::new(0.0, 1.0, 0.0),
            vfov: camera_data.vfov,
            aperture: camera_data.aperture,
            focus_distance: camera_data.focus_distance,
            aspect_ratio: image.aspect_ratio,
            time_a: 0.0,
            time_b: 1.0,
        },
        &image,
    );
    let mut shared_camera = camera.get_shared_camera();

    let mut scene = Scene::new(
        camera.get_shared_camera(),
        image.clone(),
        scene_data.objects,
    );
    let (objs, reader) = scene.get_shared_objects();
    let mut bvh = BoundingVolumeHirearchy::new(objs, reader, 0.0, 1.0);
    let (render_sender, render_receiver) = std::sync::mpsc::channel::<Result<(), TracerError>>();
    let mut window_res: Result<(), TracerError> = Ok(());
    let mut screen_buffer_res: Result<(), TracerError> = Ok(());
    let render_exit = Arc::new(SignalEvent::manual(false));
    let update_exit = Arc::clone(&render_exit);
    let image_action_signal = Arc::new(SignalEvent::manual(false));

    let renderer: Box<dyn Renderer> = (&config.renderer, &config.render, &image).into();
    let renderer_preview: Box<dyn Renderer> =
        (&config.preview_renderer, &config.preview, &image).into();

    let scene_controller = {
        match &config.scene_controller {
            config::SceneControllerConfig::Interactive => InteractiveScene::new(
                log.new(o!("scope" => "scene-controller")),
                config.clone(),
                image.clone(),
                camera_data,
                renderer,
                renderer_preview,
            ),
        }
    };

    let mut inputs = KeyInputs::new();
    inputs.register_inputs(scene_controller.register_key_inputs());

    rayon::scope(|s| {
        s.spawn(|_| {
            let logger = log.new(o!("scope" => "screen-buffer"));
            while !update_exit.status() && screen_buffer_res.is_ok() {
                screen_buffer_res = screen_buffer.update();

                if screen_buffer_res.is_ok() && image_action_signal.status() {
                    image_action_signal.reset();
                    screen_buffer_res = image_action.action(screen_buffer.rgb(), &config, &logger);
                }
            }
        });

        // Render
        let scene_controller = &scene_controller; // Avoid moving the scene_controller
        let image_action_signal = Arc::clone(&image_action_signal);
        s.spawn(move |_| {
            // Seed the first image
            let mut render_res = scene_controller.render(
                true,
                &shared_camera,
                &bvh,
                background,
                &screen_buffer_writer,
                &image_action_signal,
            );

            while render_res.is_ok() {
                render_res = (!render_exit.wait_timeout(Duration::from_secs(0)))
                    .then_some(|| ())
                    .ok_or(TracerError::ExitEvent)
                    .and_then(|_| bvh.update())
                    .and_then(|_| shared_camera.update())
                    .and_then(|_| {
                        scene_controller.render(
                            shared_camera.changed() || bvh.changed(),
                            &shared_camera,
                            &bvh,
                            background,
                            &screen_buffer_writer,
                            &image_action_signal,
                        )
                    });
            }

            if render_res.is_err() {
                render_exit.signal();
            }
            let _ = render_sender.send(render_res).map_err(|e| {
                // TODO: Logging
                println!("Failed to send render result: {}", e);
            });
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
            .and_then(|mut window| {
                let mut t = Instant::now();
                let mut res: Result<(), TracerError> = Ok(());
                while res.is_ok()
                    && window.is_open()
                    && !window.is_key_down(Key::Escape)
                    && !update_exit.status()
                {
                    let dt = t.elapsed().as_micros() as f64;
                    t = Instant::now();
                    res = inputs
                        .update(&mut window)
                        .and_then(|_| scene.update())
                        .and_then(|_| camera.update())
                        .and_then(|_| inputs.get_presses())
                        .and_then(|key_presses| {
                            scene_controller.update(
                                dt,
                                key_presses,
                                inputs.get_mouse_pos(&mut window),
                                &mut camera,
                                &mut scene,
                            )
                        })
                        .and_then(|_| image_buffer.update())
                        .and_then(|_| image_buffer_reader.update())
                        .and_then(|_| {
                            if image_buffer_reader.changed() {
                                for (i, c) in image_buffer_reader.rgb().iter().enumerate() {
                                    let red: u32 = (c.x() * 255.0) as u32;
                                    let green: u32 = (c.y() * 255.0) as u32;
                                    let blue: u32 = (c.z() * 255.0) as u32;
                                    // XRGB
                                    screen_data_buffer[i] =
                                        (255 << 24) | (red << 16) | green << 8 | blue;
                                }
                                window
                                    .update_with_buffer(
                                        &screen_data_buffer,
                                        image.width,
                                        image.height,
                                    )
                                    .map_err(|e| TracerError::FailedToUpdateWindow(e.to_string()))
                            } else {
                                window.update();
                                Ok(())
                            }
                        })
                }

                update_exit.signal();
                scene_controller.stop();
                res
            });
        })
    });

    // Could technically be more but we only expect a single one.
    let render_res = render_receiver
        .recv_timeout(Duration::from_millis(0))
        .map_err(|e| {
            TracerError::RecieveError(format!("Render thread did not produce a result: {}", e))
        })?;
    screen_buffer_res.and(window_res.and(render_res))
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

// There is a problem with slog where if the log is created in the
// same scope as where a process::exit is called it won't flush
// correctly before exiting.
fn bridge_main(config: Config) -> i32 {
    let log_file = std::env::temp_dir().join("racer-tracer.log");
    let log = create_log(log_file.as_ref()).expect("Expected to be able to create a log");
    let term = Terminal::new(log.new(o!("scope" => "terminal")));
    terminal::write_term!(term, &format!("Log file: {}", log_file.display()));

    match run(config, log.new(o!("scope" => "run")), term) {
        Err(TracerError::ExitEvent) => 0,
        Ok(_) => 0,
        Err(e) => {
            error!(log, "Error: {}", e);
            let exit_code = i32::from(e);
            error!(log, "Exiting with: {}", exit_code);
            exit_code
        }
    }
}
fn main() {
    match Config::try_from(Args::from_args()).map(bridge_main) {
        Ok(ec) => std::process::exit(ec),
        Err(e) => {
            println!("Failed to parse config file: {}", e);
            std::process::exit(0)
        }
    }
}
