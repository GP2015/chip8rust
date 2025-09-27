use serde::Deserialize;
use serde_with::serde_as;
use std::fs;
use toml;

const CONFIG_FILE_PATH: &str = "config.toml";

#[serde_as]
#[derive(Deserialize)]
pub struct Config {
    // CPU Settings:
    instructions_per_second: usize,
    use_new_shift_instruction: bool,
    use_new_jump_instruction: bool,
    set_flag_for_index_overflow: bool,
    move_index_with_reads: bool,
    use_true_randomness: bool,
    fake_randomness_seed: u64,
    allow_program_counter_overflow: bool,

    // GPU Settings:
    horizontal_resolution: usize,
    vertical_resolution: usize,
    wrap_pixels: bool,
    render_occasion: String,
    render_frequency: f64,

    // RAM Settings:
    font_start_index_on_heap: u16,
    stack_size: usize,
    allow_stack_overflow: bool,
    #[serde_as(as = "[_; 80]")]
    font_data: [u8; 80],

    // Timer settings:
    delay_timer_decrement_rate: f64,
    sound_timer_decrement_rate: f64,

    // Input Settings:
    key_bindings: [String; 16],
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
