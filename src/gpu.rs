use crate::config::{GPUConfig, RenderOccasion};
use crate::emulib::Limiter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use winit::dpi::LogicalSize;

pub struct GPU {
    active: Arc<AtomicBool>,
    config: GPUConfig,
    framebuffer: Mutex<Vec<bool>>,
}

impl GPU {
    pub fn try_new(active: Arc<AtomicBool>, config: GPUConfig) -> Option<Arc<Self>> {
        if config.render_frequency <= 0.0 {
            eprintln!("Error: The graphic render frequency must be greater than 0.");
            active.store(false, Ordering::Relaxed);
            return None;
        }

        return Some(Arc::new(Self {
            active,
            framebuffer: Mutex::new(vec![
                false;
                config.horizontal_resolution as usize
                    * config.vertical_resolution as usize
            ]),
            config,
        }));
    }

    pub fn get_window_size(&self) -> LogicalSize<u32> {
        return LogicalSize::new(
            self.config.horizontal_resolution,
            self.config.vertical_resolution,
        );
    }

    pub fn render_separately(&self) -> bool {
        return matches!(self.config.render_occasion, RenderOccasion::Frequency);
    }

    pub fn run_separate_render(&self) {
        let mut limiter = Limiter::new(self.config.render_frequency, true);

        while self.active.load(Ordering::Relaxed) {
            limiter.wait_if_early();

            // To do
        }
    }
}
