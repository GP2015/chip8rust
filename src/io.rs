use crate::config::{IOConfig, RenderOccasion};
use crate::emulib::Limiter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use winit::event_loop::EventLoop;

const RESOLUTION_SCALAR: i32 = 10;
const WINDOW_TITLE: &str = "CHIP-8 Emulator";

pub struct IO {
    active: Arc<AtomicBool>,
    config: IOConfig,
    framebuffer: Mutex<Vec<bool>>,
}

impl IO {
    pub fn try_new(active: Arc<AtomicBool>, config: IOConfig) -> Option<Arc<Self>> {
        if config.render_frequency <= 0.0 {
            eprintln!("Error: The IO Manager's render frequency must be greater than 0.");
            active.store(false, Ordering::Relaxed);
            return None;
        }

        return Some(Arc::new(Self {
            active,
            framebuffer: Mutex::new(vec![
                false;
                config.horizontal_resolution
                    * config.vertical_resolution
            ]),
            config,
        }));
    }

    fn separate_render(self) {
        let mut limiter = Limiter::new(self.config.render_frequency, true);

        while self.active.load(Ordering::Relaxed) {
            limiter.wait_if_early();
        }
    }

    pub fn run(&self) {
        let render_handle = match self.config.render_occasion {
            RenderOccasion::Frequency => Some(thread::spawn(move || this.separate_render())),
            _ => None,
        };

        let event_loop = EventLoop::new().unwrap();
    }
}

struct SeparateRender {
    active: Arc<AtomicBool>,
}

impl SeparateRender {
    pub fn run() {}
}
