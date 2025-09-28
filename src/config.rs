use serde::Deserialize;
use serde_with::serde_as;
use std::fs;
use toml;

const CONFIG_FILE_PATH: &str = "config.toml";

#[serde_as]
#[derive(Deserialize)]
pub struct Config {
    // CPU Settings:
    pub instructions_per_second: f64,
    pub use_new_shift_instruction: bool,
    pub use_new_jump_instruction: bool,
    pub set_flag_for_index_overflow: bool,
    pub move_index_with_reads: bool,
    pub allow_program_counter_overflow: bool,
    pub use_true_randomness: bool,
    pub fake_randomness_seed: u64,

    // GPU Settings:
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
    pub wrap_pixels: bool,
    pub render_occasion: String,
    pub render_frequency: f64,

    // RAM Settings:
    pub stack_size: usize,
    pub allow_stack_overflow: bool,
    pub allow_heap_overflow: bool,
    pub font_start_index_on_heap: u16,
    #[serde_as(as = "[_; 80]")]
    pub font_data: [u8; 80],

    // Timer settings:
    pub delay_timer_decrement_rate: f64,
    pub sound_timer_decrement_rate: f64,

    // Input Settings:
    pub key_bindings: [String; 16],
}

pub fn generate_config() -> Option<Config> {
    let Ok(raw_config) = fs::read_to_string(CONFIG_FILE_PATH) else {
        eprintln!("Error: Could not find config.toml.");
        return None;
    };

    let Ok(config) = toml::from_str(&raw_config) else {
        eprintln!("Error: Could not parse config.toml.");
        return None;
    };

    return Some(config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_config() {
        let _ = generate_config().unwrap();
    }
}
