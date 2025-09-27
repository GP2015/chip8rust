use std::fs;
use std::sync::Mutex;

const PROGRAM_START_INDEX: usize = 0x200;
const HEAP_SIZE: usize = 0x1000;

pub struct RAM {
    heap: Mutex<[u8; HEAP_SIZE]>,
}

impl RAM {
    pub fn new() -> Self {
        Self {
            heap: Mutex::new([0; HEAP_SIZE]),
        }
    }

    pub fn load_program(&self, program_path: &String) -> bool {
        let Ok(program) = fs::read(&program_path) else {
            eprintln!("Error: Could not find valid program at {program_path}.");
            return false;
        };

        let mut heap = self.heap.lock().unwrap();

        if PROGRAM_START_INDEX + program.len() > heap.len() {
            eprintln!("Error: Program {program_path} is too large to fit in the heap.");
            return false;
        }

        heap[PROGRAM_START_INDEX..PROGRAM_START_INDEX + program.len()].copy_from_slice(&program);

        return true;
    }
}
