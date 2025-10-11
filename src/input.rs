use crate::config::InputConfig;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct InputManager {
    active: Arc<AtomicBool>,
    config: InputConfig,
}

impl InputManager {
    pub fn try_new(active: Arc<AtomicBool>, config: InputConfig) -> Option<Arc<Self>> {
        return Some(Arc::new(Self { active, config }));
    }
}
