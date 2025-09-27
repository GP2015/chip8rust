use crate::config::Config;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const PROGRAM_START_INDEX: u16 = 0x200;
pub const HEAP_SIZE: usize = 0x1000;

pub struct RAM {
    active: Arc<AtomicBool>,
    config: Arc<Config>,
    heap: Mutex<[u8; HEAP_SIZE]>,
}

impl RAM {
    pub fn new(active: Arc<AtomicBool>, config: Arc<Config>) -> Self {
        Self {
            active,
            config,
            heap: Mutex::new([0; HEAP_SIZE]),
        }
    }

    pub fn load_program(&self, program_path: &String) -> bool {
        let Ok(program) = fs::read(&program_path) else {
            eprintln!("Error: Could not find valid program at {program_path}.");
            return false;
        };

        let mut heap = self.heap.lock().unwrap();

        if usize::from(PROGRAM_START_INDEX) + program.len() > heap.len() {
            eprintln!("Error: Program {program_path} is too large to fit in the heap.");
            return false;
        }

        heap[usize::from(PROGRAM_START_INDEX)..usize::from(PROGRAM_START_INDEX) + program.len()]
            .copy_from_slice(&program);

        return true;
    }

    pub fn get_instruction(&self, addr: u16) -> Option<u16> {
        if addr == 0xFFF {
            if self.config.allow_program_counter_overflow {
                let heap = self.heap.lock().unwrap();
                return Some((u16::from(heap[0xFFF]) << 8) | u16::from(heap[0x000]));
            } else {
                eprintln!("Error: Program counter overflowed.");
                return None;
            }
        }

        let heap = self.heap.lock().unwrap();
        let addr = usize::from(addr);

        return Some((u16::from(heap[addr]) << 8) | u16::from(heap[addr + 1]));
    }
}
