use crate::config::{DelayTimerConfig, SoundTimerConfig};
use crate::emulib::Limiter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct DelayTimer {
    active: Arc<AtomicBool>,
    config: DelayTimerConfig,
    value: Mutex<u8>,
}

impl DelayTimer {
    pub fn try_new(active: Arc<AtomicBool>, config: DelayTimerConfig) -> Option<Arc<Self>> {
        if config.delay_timer_decrement_rate <= 0.0 {
            eprintln!("Error: The delay timer's decrement rate must be greater than zero.");
            active.store(false, Ordering::Relaxed);
            return None;
        }

        return Some(Arc::new(Self {
            active,
            config,
            value: Mutex::new(0),
        }));
    }

    #[cfg(test)]
    pub fn new_default(active: Arc<AtomicBool>) -> Arc<Self> {
        Self::try_new(
            active,
            DelayTimerConfig {
                delay_timer_decrement_rate: 60.0,
            },
        )
        .unwrap()
    }

    pub fn run(&self) {
        let mut limiter = Limiter::new(self.config.delay_timer_decrement_rate, true);

        while self.active.load(Ordering::Relaxed) {
            limiter.wait_if_early();

            let mut value = self.value.lock().unwrap();

            if *value > 0 {
                *value -= 1;
            }
        }
    }

    pub fn get_value(&self) -> u8 {
        return *self.value.lock().unwrap();
    }

    pub fn set_value(&self, val: u8) {
        *self.value.lock().unwrap() = val;
    }
}

pub struct SoundTimer {
    active: Arc<AtomicBool>,
    config: SoundTimerConfig,
    value: Mutex<u8>,
}

impl SoundTimer {
    pub fn try_new(active: Arc<AtomicBool>, config: SoundTimerConfig) -> Option<Arc<Self>> {
        if config.sound_timer_decrement_rate <= 0.0 {
            eprintln!("Error: The sound timer's decrement rate must be greater than zero.");
            active.store(false, Ordering::Relaxed);
            return None;
        }

        return Some(Arc::new(Self {
            active,
            config,
            value: Mutex::new(0),
        }));
    }

    #[cfg(test)]
    pub fn new_default(active: Arc<AtomicBool>) -> Arc<Self> {
        Self::try_new(
            active,
            SoundTimerConfig {
                sound_timer_decrement_rate: 60.0,
            },
        )
        .unwrap()
    }

    pub fn run(&self) {
        let mut limiter = Limiter::new(self.config.sound_timer_decrement_rate, true);

        while self.active.load(Ordering::Relaxed) {
            limiter.wait_if_early();

            let mut value = self.value.lock().unwrap();

            if *value > 0 {
                *value -= 1;
            }
        }
    }

    pub fn get_value(&self) -> u8 {
        return *self.value.lock().unwrap();
    }

    pub fn set_value(&self, val: u8) {
        *self.value.lock().unwrap() = val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    fn create_delay_objects() -> (Arc<DelayTimer>, JoinHandle<()>, Arc<AtomicBool>) {
        let active = Arc::new(AtomicBool::new(true));
        let timer = DelayTimer::new_default(active.clone());
        let timer_clone = timer.clone();
        let handle = thread::spawn(move || timer_clone.run());
        return (timer, handle, active);
    }

    fn create_sound_objects() -> (Arc<SoundTimer>, JoinHandle<()>, Arc<AtomicBool>) {
        let active = Arc::new(AtomicBool::new(true));
        let timer = SoundTimer::new_default(active.clone());
        let timer_clone = timer.clone();
        let handle = thread::spawn(move || timer_clone.run());
        return (timer, handle, active);
    }

    #[test]
    fn test_delay_timer_decrement() {
        let (timer, handle, active) = create_delay_objects();

        timer.set_value(5);

        thread::sleep(Duration::from_millis(150));

        assert_eq!(0, timer.get_value());
        assert!(active.load(Ordering::Relaxed));

        active.store(false, Ordering::Relaxed);
        handle.join().unwrap();
    }

    #[test]
    fn test_sound_timer_decrement() {
        let (timer, handle, active) = create_sound_objects();

        timer.set_value(5);

        thread::sleep(Duration::from_millis(150));

        assert_eq!(0, timer.get_value());
        assert!(active.load(Ordering::Relaxed));

        active.store(false, Ordering::Relaxed);
        handle.join().unwrap();
    }
}
