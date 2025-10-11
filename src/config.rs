use serde::Deserialize;
use serde_with::serde_as;
use std::fs;
use toml;

const CONFIG_FILE_PATH: &str = "config.toml";

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub cpu: CPUConfig,
    pub gpu: GPUConfig,
    pub input: InputConfig,
    pub ram: RAMConfig,
    pub delay_timer: DelayTimerConfig,
    pub sound_timer: SoundTimerConfig,
}

#[derive(Deserialize, Debug)]
pub struct CPUConfig {
    pub instructions_per_second: f64,
    pub use_new_shift_instruction: bool,
    pub use_new_jump_instruction: bool,
    pub set_flag_for_index_overflow: bool,
    pub move_index_with_reads: bool,
    pub allow_program_counter_overflow: bool,
    pub use_true_randomness: bool,
    pub fake_randomness_seed: u64,
    pub allow_index_register_overflow: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RenderOccasion {
    Changes,
    Frequency,
}

#[derive(Deserialize, Debug)]
pub struct GPUConfig {
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub wrap_pixels: bool,
    pub render_occasion: RenderOccasion,
    pub render_frequency: f64,
}

#[derive(Deserialize, Debug)]
pub struct InputConfig {
    pub key_bindings: [String; 16],
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct RAMConfig {
    pub stack_size: usize,
    pub allow_stack_overflow: bool,
    pub allow_heap_overflow: bool,
    pub font_starting_address: u16,
    #[serde_as(as = "[_; 80]")]
    pub font_data: [u8; 80],
}

#[derive(Deserialize, Debug)]
pub struct DelayTimerConfig {
    pub delay_timer_decrement_rate: f64,
}

#[derive(Deserialize, Debug)]
pub struct SoundTimerConfig {
    pub sound_timer_decrement_rate: f64,
}

pub fn generate_configs() -> Option<Config> {
    let Ok(raw_config) = fs::read_to_string(CONFIG_FILE_PATH) else {
        eprintln!("Error: Could not find config.toml.");
        return None;
    };

    let Ok(config): Result<Config, _> = toml::from_str(&raw_config) else {
        eprintln!("Error: Could not parse config.toml.");
        return None;
    };

    return Some(config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_configs() {
        let _ = generate_configs().unwrap();
    }
}
