mod config;
mod cpu;
mod emulib;
mod instructions;
mod io;
mod ram;
mod timer;

use crate::cpu::CPU;
use crate::io::IO;
use crate::ram::RAM;
use crate::timer::{DelayTimer, SoundTimer};
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

    let Some(ram) = RAM::try_new(active.clone(), config.ram) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let Some(delay_timer) = DelayTimer::try_new(active.clone(), config.delay_timer) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let Some(sound_timer) = SoundTimer::try_new(active.clone(), config.sound_timer) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let Some(io) = IO::try_new(active.clone(), config.io) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let Some(cpu) = CPU::try_new(
        active.clone(),
        config.cpu,
        ram.clone(),
        delay_timer.clone(),
        sound_timer.clone(),
    ) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    if !ram.load_program(&args.program_path) {
        eprintln!("Emulator terminated with error.");
        return;
    }

    let delay_timer_handle = thread::spawn(move || delay_timer.run());
    let sound_timer_handle = thread::spawn(move || sound_timer.run());
    let io_handle = thread::spawn(move || io.run());
    let cpu_handle = thread::spawn(move || cpu.run());

    delay_timer_handle.join().unwrap();
    sound_timer_handle.join().unwrap();
    cpu_handle.join().unwrap();

    println!("Stopping emulator...")
}
