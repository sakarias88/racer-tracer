use minifb::{Key, MouseButton, MouseMode, Window};

use crate::data_bus::{DataBus, DataReader, DataWriter};
use crate::error::TracerError;

pub enum ListenKeyEvents {
    Release(Vec<Key>),
    Down(Vec<Key>),
    MouseMove(MouseButton),
}

#[derive(Clone)]
pub enum KeyEvent {
    Released(Key),
    Down(Key),
    MouseDelta(MouseButton, f64, f64),
}

pub struct MousePos {
    pub x: f64,
    pub y: f64,
}

pub struct Mouse {
    move_on_press: MouseButton,
    mouse_down: bool,
    delta: MousePos,
}

impl Mouse {
    pub fn new(key: MouseButton) -> Self {
        Self {
            move_on_press: key,
            mouse_down: false,
            delta: MousePos { x: 0.0, y: 0.0 },
        }
    }

    pub fn update(&mut self, window: &mut Window) -> Option<KeyEvent> {
        let mut res = None;
        if window.get_mouse_down(self.move_on_press) {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Pass) {
                if !self.mouse_down {
                    self.delta.x = x as f64;
                    self.delta.y = y as f64;
                }
                res = Some(KeyEvent::MouseDelta(
                    self.move_on_press,
                    self.delta.x - x as f64,
                    self.delta.y - y as f64,
                ));

                self.delta.x = x as f64;
                self.delta.y = y as f64;
            }

            self.mouse_down = true;
        } else {
            self.mouse_down = false;
        }

        res
    }
}

pub struct KeyInputs {
    bus: DataBus<KeyEvent>,
    key_writer: DataWriter<KeyEvent>,
    key_reader: DataReader<KeyEvent>,
    listen_is_down: Vec<Key>,
    listen_is_released: Vec<Key>,
    listen_mouse: Vec<Mouse>,
}

impl<'window> KeyInputs {
    pub fn new() -> Self {
        let mut bus = DataBus::new("key-inputs");
        KeyInputs {
            key_reader: bus.get_reader(),
            key_writer: bus.get_writer(),
            bus,
            listen_is_down: vec![],
            listen_is_released: vec![],
            listen_mouse: vec![],
        }
    }

    pub fn register_inputs(&mut self, inputs: Vec<ListenKeyEvents>) {
        inputs.into_iter().for_each(|input| match input {
            ListenKeyEvents::Release(mut keys) => self.listen_is_released.append(&mut keys),
            ListenKeyEvents::Down(mut keys) => self.listen_is_down.append(&mut keys),
            ListenKeyEvents::MouseMove(mouse_key) => {
                self.listen_mouse.push(Mouse::new(mouse_key));
            }
        });
    }

    // This just gets all available presses. If you want to control
    // how you block etc you will have to use get_key_reader.
    pub fn get_presses(&mut self) -> Result<Vec<KeyEvent>, TracerError> {
        self.key_reader.get_messages()
    }

    #[allow(dead_code)]
    pub fn get_key_reader(&mut self) -> DataReader<KeyEvent> {
        self.bus.get_reader()
    }

    pub fn get_mouse_pos(&self, window: &'window mut Window) -> Option<MousePos> {
        window
            .get_mouse_pos(MouseMode::Pass)
            .map(|(x, y)| MousePos {
                x: x as f64,
                y: y as f64,
            })
    }

    pub fn update(&mut self, window: &'window mut Window) -> Result<(), TracerError> {
        self.bus.update()?;
        if window.is_active() {
            self.listen_mouse
                .iter_mut()
                .map(|mouse| mouse.update(window))
                .filter(|v| v.is_some())
                .try_for_each(|v| self.key_writer.write(v.expect("some value to be some")))?;

            self.listen_is_down
                .iter()
                .filter(|key| window.is_key_down(**key))
                .try_for_each(|key| self.key_writer.write(KeyEvent::Down(*key)))?;

            self.listen_is_released
                .iter()
                .filter(|key| window.is_key_released(**key))
                .try_for_each(|key| self.key_writer.write(KeyEvent::Released(*key)))
        } else {
            Ok(())
        }
    }
}
