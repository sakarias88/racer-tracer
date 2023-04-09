use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

use minifb::Key;
use slog::Logger;
use synchronoise::SignalEvent;

use crate::{
    camera::Camera,
    config::Config,
    error::TracerError,
    image::Image,
    image_action::ImageAction,
    key_inputs::{KeyCallback, KeyEvent, KeyInputs},
    renderer::{RenderData, Renderer},
    scene::Scene,
    terminal::Terminal,
};

use super::{create_screen_buffer, SceneController, SceneData};

pub struct InteractiveScene<'renderer, 'action> {
    screen_buffer: RwLock<Vec<u32>>,
    preview_buffer: RwLock<Vec<u32>>,
    camera_speed: f64,
    render_image_event: SignalEvent,
    buffer_updated: SignalEvent,
    stop_event: SignalEvent,

    log: Logger,
    term: Terminal,
    image_action: &'action dyn ImageAction,
    config: Config,
    scene: Scene,
    camera: RwLock<Camera>,
    image: Image,
    renderer: &'renderer dyn Renderer,
    renderer_preview: &'renderer dyn Renderer,
}

impl<'renderer, 'action> InteractiveScene<'renderer, 'action> {
    pub fn new(scene_data: SceneData, camera_speed: f64) -> Self {
        Self {
            screen_buffer: RwLock::new(create_screen_buffer(&scene_data.image)),
            preview_buffer: RwLock::new(create_screen_buffer(&scene_data.image)),
            camera_speed,
            render_image_event: SignalEvent::manual(false),
            buffer_updated: SignalEvent::manual(false),
            stop_event: SignalEvent::manual(false),

            log: scene_data.log,
            term: scene_data.term,
            image_action: (&scene_data.config.image_action).into(),
            scene: scene_data.scene,
            camera: RwLock::new(scene_data.camera),
            image: scene_data.image,
            renderer: (&scene_data.config.renderer).into(),
            renderer_preview: (&scene_data.config.preview_renderer).into(),
            config: scene_data.config,
        }
    }
}

impl<'renderer, 'action> SceneController for InteractiveScene<'renderer, 'action> {
    fn get_inputs(&self) -> Vec<KeyCallback> {
        vec![
            KeyInputs::input(KeyEvent::Release, Key::R, |_| {
                self.render_image_event.signal();
                Ok(())
            }),
            KeyInputs::input(KeyEvent::Down, Key::W, |dt| {
                self.camera
                    .write()
                    .map_err(|e| TracerError::KeyError(e.to_string()))
                    .map(|mut cam| {
                        cam.go_forward(-dt * self.camera_speed);
                    })
            }),
            KeyInputs::input(KeyEvent::Down, Key::S, |dt| {
                self.camera
                    .write()
                    .map_err(|e| TracerError::KeyError(e.to_string()))
                    .map(|mut cam| {
                        cam.go_forward(dt * self.camera_speed);
                    })
            }),
            KeyInputs::input(KeyEvent::Down, Key::A, |dt| {
                self.camera
                    .write()
                    .map_err(|e| TracerError::KeyError(e.to_string()))
                    .map(|mut cam| {
                        cam.go_right(-dt * self.camera_speed);
                    })
            }),
            KeyInputs::input(KeyEvent::Down, Key::D, |dt| {
                self.camera
                    .write()
                    .map_err(|e| TracerError::KeyError(e.to_string()))
                    .map(|mut cam| {
                        cam.go_right(dt * self.camera_speed);
                    })
            }),
        ]
    }

    fn get_buffer(&self) -> Result<Option<Vec<u32>>, TracerError> {
        self.buffer_updated
            .wait_timeout(Duration::from_secs(0))
            .then_some(|| ())
            .map_or(Ok(None), |_| {
                self.screen_buffer
                    .read()
                    .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
                    .map(|v| Some(v.to_owned()))
            })
    }

    fn render(&self) -> Result<(), TracerError> {
        self.render_image_event
            .wait_timeout(Duration::from_secs(0))
            .then_some(|| ())
            .map_or_else(
                || {
                    // Render preview
                    self.render_image_event.reset();

                    // We do not want partial screen updates for the preview.
                    // Which is why we send in a different buffer.
                    self.renderer_preview
                        .render(RenderData {
                            buffer: &self.preview_buffer,
                            camera: &self.camera,
                            image: &self.image,
                            scene: &self.scene,
                            config: &self.config,
                            cancel_event: None,
                            buffer_updated: None,
                        })
                        .and_then(|_| {
                            self.screen_buffer
                                .write()
                                .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
                        })
                        .and_then(|w_buffer| {
                            self.preview_buffer
                                .read()
                                .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
                                .map(|r_buffer| (r_buffer, w_buffer))
                        })
                        .map(|(r_buffer, mut w_buffer)| {
                            // For the preview we want the complete image
                            // result before signaling the image is done.
                            *w_buffer = r_buffer.to_owned();
                            self.buffer_updated.signal();
                        })
                },
                |_| {
                    let render_time = Instant::now();
                    self.render_image_event.reset();

                    // When we render the final image we want partial
                    // updates to the screen buffer. We send in our
                    // original screen_buffer and the buffer_updated
                    // signal. This will ensure that the window will
                    // get updated with a new buffer as soon as a
                    // thread finishes writing a block and we get
                    // partial updates of the rendered image.
                    self.renderer
                        .render(RenderData {
                            buffer: &self.screen_buffer,
                            camera: &self.camera,
                            image: &self.image,
                            scene: &self.scene,
                            config: &self.config,
                            cancel_event: Some(&self.render_image_event),
                            buffer_updated: Some(&self.buffer_updated),
                        })
                        .and_then(|_| {
                            if !self.render_image_event.status() {
                                info!(
                                    self.log,
                                    "It took {} seconds to render the image.",
                                    Instant::now().duration_since(render_time).as_secs()
                                );
                            } else {
                                info!(self.log, "Image render cancelled.");
                            }

                            self.image_action.action(
                                &self.screen_buffer,
                                &self.stop_event,
                                &self.render_image_event,
                                &self.config,
                                self.log.new(o!("scope" => "image-action")),
                                &self.term,
                            )
                        })
                },
            )
    }

    fn stop(&self) {
        // If we are currently rendering anything we try to cancel it
        self.render_image_event.signal();
        self.stop_event.signal();
    }
}
