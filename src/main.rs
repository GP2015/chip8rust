mod comps;
mod config;
mod cpu;
mod gpu;
mod input;
mod ram;
mod timer;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    program_path: String,
}

fn main() {
    println!("Starting emulator...");

    let args = Args::parse();

    let Some(config) = config::generate_config() else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let mut components = comps::Components {
        config: config,
        ram: ram::RAM::new(),
    };

    if !components.ram.load_program(&args.program_path) {
        eprintln!("Program terminated with error.");
        return;
    }

    println!("Stopping emulator...")
}
