use serde::Deserialize;
use serde_with::serde_as;
use std::fs;
use toml;

use crate::gpu;

const CONFIG_FILE_PATH: &str = "config.toml";

// #[serde_as]
// #[derive(Deserialize)]
// struct Config {
//     // CPU Settings:
//     pub instructions_per_second: f64,
//     pub use_new_shift_instruction: bool,
//     pub use_new_jump_instruction: bool,
//     pub set_flag_for_index_overflow: bool,
//     pub move_index_with_reads: bool,
//     pub allow_program_counter_overflow: bool,
//     pub use_true_randomness: bool,
//     pub fake_randomness_seed: u64,

//     // GPU Settings:
//     pub horizontal_resolution: usize,
//     pub vertical_resolution: usize,
//     pub wrap_pixels: bool,
//     pub render_occasion: String,
//     pub render_frequency: f64,

//     // RAM Settings:
//     pub stack_size: usize,
//     pub allow_stack_overflow: bool,
//     pub allow_heap_overflow: bool,
//     pub font_start_index_on_heap: u16,
//     #[serde_as(as = "[_; 80]")]
//     pub font_data: [u8; 80],

//     // Timer settings:
//     pub delay_timer_decrement_rate: f64,
//     pub sound_timer_decrement_rate: f64,

//     // Input Settings:
//     pub key_bindings: [String; 16],
// }

#[serde_as]
#[derive(Deserialize)]
pub struct Config {
    pub cpu: CPUConfig,
    pub gpu: GPUConfig,
    pub ram: RAMConfig,
    pub timer: TimerConfig,
    pub input: InputConfig,
}

#[derive(Deserialize)]
pub struct CPUConfig {
    pub instructions_per_second: f64,
    pub use_new_shift_instruction: bool,
    pub use_new_jump_instruction: bool,
    pub set_flag_for_index_overflow: bool,
    pub move_index_with_reads: bool,
    pub allow_program_counter_overflow: bool,
    pub use_true_randomness: bool,
    pub fake_randomness_seed: u64,
}

#[derive(Deserialize)]
pub struct GPUConfig {
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
    pub wrap_pixels: bool,
    pub render_occasion: String,
    pub render_frequency: f64,
}

#[serde_as]
#[derive(Deserialize)]
pub struct RAMConfig {
    pub stack_size: usize,
    pub allow_stack_overflow: bool,
    pub allow_heap_overflow: bool,
    pub font_start_index_on_heap: u16,
    #[serde_as(as = "[_; 80]")]
    pub font_data: [u8; 80],
}

#[derive(Deserialize)]
pub struct TimerConfig {
    pub delay_timer_decrement_rate: f64,
    pub sound_timer_decrement_rate: f64,
}

#[derive(Deserialize)]
pub struct InputConfig {
    pub key_bindings: [String; 16],
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

    // let cpu_cfg = CPUConfig {
    //     instructions_per_second: config.instructions_per_second,
    //     use_new_shift_instruction: config.use_new_shift_instruction,
    //     use_new_jump_instruction: config.use_new_jump_instruction,
    //     set_flag_for_index_overflow: config.set_flag_for_index_overflow,
    //     move_index_with_reads: config.move_index_with_reads,
    //     allow_program_counter_overflow: config.allow_program_counter_overflow,
    //     use_true_randomness: config.use_true_randomness,
    //     fake_randomness_seed: config.fake_randomness_seed,
    // };

    // let gpu_cfg = GPUConfig {
    //     horizontal_resolution: config.horizontal_resolution,
    //     vertical_resolution: config.vertical_resolution,
    //     wrap_pixels: config.wrap_pixels,
    //     render_occasion: config.render_occasion,
    //     render_frequency: config.render_frequency,
    // };

    // let ram_cfg = RAMConfig {
    //     stack_size: config.stack_size,
    //     allow_stack_overflow: config.allow_stack_overflow,
    //     allow_heap_overflow: config.allow_heap_overflow,
    //     font_start_index_on_heap: config.font_start_index_on_heap,
    //     font_data: config.font_data,
    // };

    // let timer_cfg = TimerConfig {
    //     delay_timer_decrement_rate: config.delay_timer_decrement_rate,
    //     sound_timer_decrement_rate: config.sound_timer_decrement_rate,
    // };

    // let input_cfg = InputConfig {
    //     key_bindings: config.key_bindings,
    // };

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
