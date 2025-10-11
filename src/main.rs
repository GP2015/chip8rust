mod config;
mod cpu;
mod emulib;
mod gpu;
mod input;
mod instructions;
mod ram;
mod timer;
mod window;

use crate::cpu::CPU;
use crate::gpu::GPU;
use crate::input::InputManager;
use crate::ram::RAM;
use crate::timer::{DelayTimer, SoundTimer};
use crate::window::WindowManager;
use clap::Parser;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use winit::event_loop::{ControlFlow, EventLoop};

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

    let Some(delay_timer) = DelayTimer::try_new(active.clone(), config.delay_timer) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let Some(sound_timer) = SoundTimer::try_new(active.clone(), config.sound_timer) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let Some(ram) = RAM::try_new(active.clone(), config.ram) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let Some(gpu) = GPU::try_new(active.clone(), config.gpu) else {
        eprintln!("Emulator terminated with error.");
        return;
    };

    let Some(input_manager) = InputManager::try_new(active, config.input) else {
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

    let window_manager = WindowManager::new(active, gpu, input_manager);

    if !ram.load_program(&args.program_path) {
        eprintln!("Emulator terminated with error.");
        return;
    }

    let mut handles = Vec::new();

    handles.push(thread::spawn(move || delay_timer.run()));
    handles.push(thread::spawn(move || sound_timer.run()));

    if gpu.render_separately() {
        let gpu_clone = gpu.clone();
        handles.push(thread::spawn(move || gpu_clone.run_separate_render()));
    }

    handles.push(thread::spawn(move || cpu.run()));

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(io);

    io.run();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Stopping emulator...");
}
