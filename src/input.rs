use crate::config::InputConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use winit::keyboard::Key;
use winit::keyboard::SmolStr;
use winit_input_helper::WinitInputHelper;

const NUMBER_OF_INPUTS: usize = 16;

pub struct InputManager {
    active: Arc<AtomicBool>,
    config: InputConfig,
    key_states: Mutex<[bool; 16]>,
}

impl InputManager {
    pub fn try_new(active: Arc<AtomicBool>, config: InputConfig) -> Option<Arc<Self>> {
        return Some(Arc::new(Self {
            active,
            config,
            key_states: Mutex::new([false; 16]),
        }));
    }

    #[cfg(test)]
    pub fn new_default(active: Arc<AtomicBool>) -> Arc<Self> {
        Self::try_new(
            active,
            InputConfig {
                key_bindings: [
                    Key::Character(SmolStr::new("1")),
                    Key::Character(SmolStr::new("2")),
                    Key::Character(SmolStr::new("3")),
                    Key::Character(SmolStr::new("q")),
                    Key::Character(SmolStr::new("w")),
                    Key::Character(SmolStr::new("e")),
                    Key::Character(SmolStr::new("a")),
                    Key::Character(SmolStr::new("s")),
                    Key::Character(SmolStr::new("d")),
                    Key::Character(SmolStr::new("x")),
                    Key::Character(SmolStr::new("z")),
                    Key::Character(SmolStr::new("c")),
                    Key::Character(SmolStr::new("4")),
                    Key::Character(SmolStr::new("r")),
                    Key::Character(SmolStr::new("f")),
                    Key::Character(SmolStr::new("v")),
                ],
            },
        )
        .unwrap()
    }

    pub fn update_input(&self, input: &WinitInputHelper) {
        let mut key_states = self.key_states.lock().unwrap();

        for i in 0..NUMBER_OF_INPUTS {
            if input.key_pressed_logical(self.config.key_bindings[i].as_ref()) {
                key_states[i] = true;
            } else if input.key_released_logical(self.config.key_bindings[i].as_ref()) {
                key_states[i] = false;
            }
        }
    }

    pub fn get_key_state(&self, key_index: u8) -> bool {
        if cfg!(debug_assertions) && key_index > 0xF {
            panic!("Error: Should not be possible to read non-existent key_states.");
        }

        return self.key_states.lock().unwrap()[key_index as usize];
    }
}
