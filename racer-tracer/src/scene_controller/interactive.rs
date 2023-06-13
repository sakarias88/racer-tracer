use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

use minifb::{Key, MouseButton};
use slog::Logger;
use synchronoise::SignalEvent;

use crate::{
    background_color::BackgroundColor,
    camera::{Camera, CameraData, SharedCamera},
    config::Config,
    error::TracerError,
    geometry::Hittable,
    image::Image,
    image_action::ImageAction,
    key_inputs::{KeyEvent, ListenKeyEvents, MousePos},
    renderer::{RenderData, Renderer},
    scene::Scene,
    terminal::Terminal,
    tone_map::ToneMap,
};

use super::{create_screen_buffer, SceneController};

pub struct InteractiveScene<'action> {
    screen_buffer: RwLock<Vec<u32>>,
    camera_speed: f64,
    camera_sensitivity: f64,
    object_move_speed: f64,
    render_image_event: SignalEvent,
    stop_event: SignalEvent,
    log: Logger,
    term: Terminal,
    image_action: &'action dyn ImageAction,
    config: Config,
    image: Image,
    renderer: Box<dyn Renderer>,
    renderer_preview: Box<dyn Renderer>,
}

impl<'action> InteractiveScene<'action> {
    pub fn new(
        log: Logger,
        term: Terminal,
        config: Config,
        image: Image,
        camera_data: CameraData,
        renderer: Box<dyn Renderer>,
        renderer_preview: Box<dyn Renderer>,
    ) -> Self {
        Self {
            screen_buffer: RwLock::new(create_screen_buffer(&image)),
            camera_speed: camera_data.speed,
            camera_sensitivity: camera_data.sensitivity,
            object_move_speed: 0.000001,
            render_image_event: SignalEvent::manual(false),
            stop_event: SignalEvent::manual(false),
            log,
            term,
            image_action: (&config.image_action).into(),
            image,
            renderer,
            renderer_preview,
            config,
        }
    }
}

impl<'action> SceneController for InteractiveScene<'action> {
    fn update(
        &self,
        dt: f64,
        keys: Vec<KeyEvent>,
        mouse_pos: Option<MousePos>,
        camera: &mut Camera,
        scene: &mut Scene,
    ) -> Result<(), TracerError> {
        keys.into_iter().try_for_each(|event| match event {
            KeyEvent::Released(key) => match key {
                Key::Q => {
                    if let Some(mp) = mouse_pos.as_ref() {
                        scene.select_object(mp.x, mp.y);
                    }
                    Ok(())
                }
                Key::E => {
                    if let Some(cookie) = scene.selected_object().as_ref() {
                        let _ = scene.remove_object(cookie);
                    }
                    Ok(())
                }
                Key::R => {
                    self.render_image_event.signal();
                    Ok(())
                }
                Key::NumPadMinus => camera.set_fov(camera.get_vfov() + 1.0),
                Key::NumPadPlus => camera.set_fov(camera.get_vfov() - 1.0),
                Key::NumPad8 => camera.set_aperture(camera.get_aperture() + 0.01),
                Key::NumPad2 => camera.set_aperture(camera.get_aperture() - 0.01),
                Key::NumPad4 => camera.set_focus_distance(camera.get_focus_distance() + 1.0),
                Key::NumPad6 => camera.set_focus_distance(camera.get_focus_distance() - 1.0),
                _ => Ok(()),
            },
            KeyEvent::Down(key) => match key {
                Key::Left => {
                    if let Some(cookie) = scene.selected_object().as_ref() {
                        let _ = scene.get_pos(cookie).and_then(|mut pos| {
                            pos.add(camera.right() * -dt * self.object_move_speed);
                            scene.set_pos(cookie, pos)
                        });
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
                Key::Right => {
                    if let Some(cookie) = scene.selected_object().as_ref() {
                        let _ = scene.get_pos(cookie).and_then(|mut pos| {
                            pos.add(camera.right() * dt * self.object_move_speed);
                            scene.set_pos(cookie, pos)
                        });
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
                Key::Up => {
                    if let Some(cookie) = scene.selected_object().as_ref() {
                        let _ = scene.get_pos(cookie).and_then(|mut pos| {
                            pos.add(camera.forward() * -dt * self.object_move_speed);
                            scene.set_pos(cookie, pos)
                        });
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
                Key::Down => {
                    if let Some(cookie) = scene.selected_object().as_ref() {
                        let _ = scene.get_pos(cookie).and_then(|mut pos| {
                            pos.add(camera.forward() * dt * self.object_move_speed);
                            scene.set_pos(cookie, pos)
                        });
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
                Key::W => camera.go_forward(-dt * self.camera_speed),
                Key::A => camera.go_right(-dt * self.camera_speed),
                Key::S => camera.go_forward(dt * self.camera_speed),
                Key::D => camera.go_right(dt * self.camera_speed),
                _ => Ok(()),
            },
            KeyEvent::MouseDelta(key, x, y) => {
                if key == MouseButton::Left {
                    camera.rotate(x * self.camera_sensitivity, y * self.camera_sensitivity)
                } else if key == MouseButton::Right {
                    if let Some(cookie) = scene.selected_object().as_ref() {
                        let _ = scene.get_pos(cookie).and_then(|mut pos| {
                            let move_delta = camera.up() * y * dt * self.object_move_speed
                                + camera.right() * -x * dt * self.object_move_speed;
                            pos.add(move_delta);
                            scene.set_pos(cookie, pos)
                        });
                        Ok(())
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
        })
    }

    fn register_key_inputs(&self) -> Vec<ListenKeyEvents> {
        vec![
            ListenKeyEvents::Release(vec![
                Key::Q,
                Key::R,
                Key::E,
                Key::NumPadMinus,
                Key::NumPadPlus,
                Key::NumPad8,
                Key::NumPad2,
                Key::NumPad4,
                Key::NumPad6,
            ]),
            ListenKeyEvents::Down(vec![
                Key::Left,
                Key::Right,
                Key::Up,
                Key::Down,
                Key::W,
                Key::A,
                Key::S,
                Key::D,
            ]),
            ListenKeyEvents::MouseMove(MouseButton::Left),
            ListenKeyEvents::MouseMove(MouseButton::Right),
        ]
    }

    fn get_buffer(&self) -> Result<Option<Vec<u32>>, TracerError> {
        self.screen_buffer
            .read()
            .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
            .map(|v| Some(v.to_owned()))
    }

    fn render(
        &self,
        scene_changed: bool,
        camera: &SharedCamera,
        scene: &dyn Hittable,
        background: &dyn BackgroundColor,
        tone_map: &dyn ToneMap,
    ) -> Result<(), TracerError> {
        if !scene_changed && !self.render_image_event.status() {
            return Ok(());
        }

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
                            camera_data: camera.data(),
                            image: &self.image,
                            scene,
                            background,
                            config: &self.config,
                            cancel_event: None,
                        })
                        .and_then(|_| {
                            self.screen_buffer
                                .write()
                                .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
                        })
                        .and_then(|w_buffer| {
                            self.renderer_preview
                                .image_data()
                                .map(|image_data| (w_buffer, image_data))
                        })
                        .map(|(mut w_buffer, image_data)| {
                            for (i, c) in image_data.rgb.into_iter().enumerate() {
                                w_buffer[i] = tone_map.tone_map(c).as_color();
                            }
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
                            camera_data: camera.data(),
                            image: &self.image,
                            scene,
                            background,
                            config: &self.config,
                            cancel_event: Some(&self.render_image_event),
                        })
                        .and_then(|_| {
                            self.screen_buffer
                                .write()
                                .map_err(|e| TracerError::FailedToAcquireLock(e.to_string()))
                        })
                        .and_then(|w_buffer| {
                            self.renderer
                                .image_data()
                                .map(|image_data| (w_buffer, image_data))
                        })
                        .map(|(mut w_buffer, image_data)| {
                            for (i, c) in image_data.rgb.into_iter().enumerate() {
                                w_buffer[i] = tone_map.tone_map(c).as_color();
                            }
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
