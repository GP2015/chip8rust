mod config;
mod cpu;
mod emulib;
mod gpu;
mod input;
mod instructions;
mod ram;
mod timer;
mod window;

use crate::config::Config;
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

struct Components {
    active: Arc<AtomicBool>,
    cpu: Arc<CPU>,
    gpu: Arc<GPU>,
    ram: Arc<RAM>,
    delay_timer: Arc<DelayTimer>,
    sound_timer: Arc<SoundTimer>,
    input_manager: Arc<InputManager>,
}

fn main() {
    println!("Starting emulator...");

    let args = Args::parse();

    let Some(comps) = create_components() else {
        eprintln!("Stopping emulator...");
        return;
    };

    let mut window_manager = WindowManager::new(
        comps.active.clone(),
        comps.gpu.clone(),
        comps.input_manager.clone(),
    );

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    if let Err(e) = event_loop.run_app(&mut window_manager) {
        eprintln!(
            "Window manager event loop failed with following error: {e}\nStopping emulator..."
        );
        return;
    };

    let mut handles = Vec::new();

    handles.push(thread::spawn(move || comps.delay_timer.run()));
    handles.push(thread::spawn(move || comps.sound_timer.run()));

    if comps.gpu.should_render_separately() {
        let gpu = comps.gpu.clone();
        handles.push(thread::spawn(move || gpu.run_separate_render()));
    }

    handles.push(thread::spawn(move || comps.cpu.run()));

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Stopping emulator...");
}

fn create_components() -> Option<Components> {
    let Some(config) = config::generate_configs() else {
        return None;
    };

    let active = Arc::new(AtomicBool::new(true));

    let Some(delay_timer) = DelayTimer::try_new(active.clone(), config.delay_timer) else {
        return None;
    };

    let Some(sound_timer) = SoundTimer::try_new(active.clone(), config.sound_timer) else {
        return None;
    };

    let Some(ram) = RAM::try_new(active.clone(), config.ram) else {
        return None;
    };

    let Some(gpu) = GPU::try_new(active.clone(), config.gpu) else {
        return None;
    };

    let Some(input_manager) = InputManager::try_new(active.clone(), config.input) else {
        return None;
    };

    let Some(cpu) = CPU::try_new(
        active.clone(),
        config.cpu,
        gpu.clone(),
        ram.clone(),
        delay_timer.clone(),
        sound_timer.clone(),
        input_manager.clone(),
    ) else {
        return None;
    };

    return Some(Components {
        active,
        cpu,
        gpu,
        ram,
        delay_timer,
        sound_timer,
        input_manager,
    });
}
