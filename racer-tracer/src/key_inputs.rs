use std::collections::HashMap;

use minifb::{Key, Window};
use slog::Logger;

use crate::error::TracerError;

pub type KeyClosure<'a> = &'a (dyn Fn(f64) -> Result<(), TracerError> + Send + Sync);

pub struct KeyInputs<'a> {
    is_down_callbacks: HashMap<Key, Vec<KeyClosure<'a>>>,
    is_released_callbacks: HashMap<Key, Vec<KeyClosure<'a>>>,
    log: Logger,
}

impl<'a> KeyInputs<'a> {
    pub fn new(log: Logger) -> Self {
        KeyInputs {
            is_down_callbacks: HashMap::new(),
            is_released_callbacks: HashMap::new(),
            log,
        }
    }

    pub fn release(&mut self, key: Key, closure: KeyClosure<'a>) {
        let callbacks = self
            .is_released_callbacks
            .entry(key)
            .or_insert_with(Vec::new);
        callbacks.push(closure);
    }

    pub fn down(&mut self, key: Key, closure: KeyClosure<'a>) {
        let callbacks = self.is_down_callbacks.entry(key).or_insert_with(Vec::new);
        callbacks.push(closure);
    }

    pub fn update(&mut self, window: &Window, dt: f64) {
        self.is_down_callbacks
            .iter_mut()
            .filter(|(key, _callbacks)| window.is_key_down(**key))
            .for_each(|(_key, callbacks)| {
                callbacks.iter_mut().for_each(|callback| {
                    if let Err(e) = callback(dt) {
                        error!(self.log, "Key callback error: {}", e);
                    }
                })
            });
        self.is_released_callbacks
            .iter_mut()
            .filter(|(key, _callbacks)| window.is_key_released(**key))
            .for_each(|(_key, callbacks)| {
                callbacks.iter_mut().for_each(|callback| {
                    if let Err(e) = callback(dt) {
                        error!(self.log, "Key callback error: {}", e);
                    }
                })
            });
    }
}
