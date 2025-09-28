mod config;
mod cpu;
mod emulib;
mod gpu;
mod input;
mod instructions;
mod ram;
mod timer;

use crate::cpu::CPU;
use crate::ram::RAM;
use clap::Parser;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    program_path: String,
}

fn main() {
    println!("Starting emulator...");

    let args = Args::parse();

    let Some(config) = config::generate_configs() else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let active = Arc::new(AtomicBool::new(true));

    let ram = Arc::new(RAM::new(active.clone(), Arc::new(config.ram)));
    let cpu = Arc::new(CPU::new(active.clone(), Arc::new(config.cpu), ram.clone()));

    if !ram.load_program(&args.program_path) {
        eprintln!("Emulator terminated with error.");
        return;
    }

    let cpu_handle = thread::spawn(move || cpu.run());

    cpu_handle.join().unwrap();

    println!("Stopping emulator...")
}
