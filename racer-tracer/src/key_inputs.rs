use std::collections::HashMap;

use minifb::{Key, Window};
use slog::Logger;

use crate::error::TracerError;

pub enum KeyEvent {
    Release,
    Down,
}

type Callback<'func> = Box<dyn Fn(f64) -> Result<(), TracerError> + Send + Sync + 'func>;
pub type KeyCallback<'func> = (KeyEvent, Key, Callback<'func>);
pub struct KeyInputs<'func> {
    is_down_callbacks: HashMap<Key, Vec<Callback<'func>>>,
    is_released_callbacks: HashMap<Key, Vec<Callback<'func>>>,
    log: Logger,
}

impl<'func, 'window> KeyInputs<'func> {
    pub fn new(log: Logger) -> Self {
        KeyInputs {
            is_down_callbacks: HashMap::new(),
            is_released_callbacks: HashMap::new(),
            log,
        }
    }

    pub fn input(
        event: KeyEvent,
        key: Key,
        callback: impl Fn(f64) -> Result<(), TracerError> + Send + Sync + 'func,
    ) -> KeyCallback<'func> {
        (event, key, Box::new(callback))
    }

    pub fn register_inputs(&mut self, inputs: Vec<KeyCallback<'func>>) {
        inputs.into_iter().for_each(|(ev, key, closure)| match ev {
            KeyEvent::Release => {
                let callbacks = self
                    .is_released_callbacks
                    .entry(key)
                    .or_insert_with(Vec::new);
                callbacks.push(closure);
            }
            KeyEvent::Down => {
                let callbacks = self.is_down_callbacks.entry(key).or_insert_with(Vec::new);
                callbacks.push(closure);
            }
        })
    }

    #[allow(dead_code)]
    pub fn release(
        &mut self,
        key: Key,
        closure: impl Fn(f64) -> Result<(), TracerError> + Send + Sync + 'func,
    ) {
        let callbacks = self
            .is_released_callbacks
            .entry(key)
            .or_insert_with(Vec::new);
        callbacks.push(Box::new(closure));
    }

    #[allow(dead_code)]
    pub fn down(
        &mut self,
        key: Key,
        closure: impl Fn(f64) -> Result<(), TracerError> + Send + Sync + 'func,
    ) {
        let callbacks = self.is_down_callbacks.entry(key).or_insert_with(Vec::new);
        callbacks.push(Box::new(closure));
    }

    pub fn update(&self, window: &'window Window, dt: f64) {
        self.is_down_callbacks
            .iter()
            .filter(|(key, _callbacks)| window.is_key_down(**key))
            .for_each(|(_key, callbacks)| {
                callbacks.iter().for_each(|callback| {
                    if let Err(e) = callback(dt) {
                        error!(self.log, "Key callback error: {}", e);
                    }
                })
            });
        self.is_released_callbacks
            .iter()
            .filter(|(key, _callbacks)| window.is_key_released(**key))
            .for_each(|(_key, callbacks)| {
                callbacks.iter().for_each(|callback| {
                    if let Err(e) = callback(dt) {
                        error!(self.log, "Key callback error: {}", e);
                    }
                })
            });
    }
}
