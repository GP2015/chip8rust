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
    stack: Mutex<Vec<u16>>,
    stack_ptr: Mutex<usize>,
}

impl RAM {
    pub fn new(active: Arc<AtomicBool>, config: Arc<Config>) -> Self {
        Self {
            active,
            heap: Mutex::new([0; HEAP_SIZE]),
            stack: Mutex::new(Vec::with_capacity(config.stack_size)),
            stack_ptr: Mutex::new(0),
            config,
        }
    }

    pub fn load_program(&self, program_path: &String) -> bool {
        let Ok(program) = fs::read(&program_path) else {
            eprintln!("Error: Could not find valid program at {program_path}.");
            self.active.store(false, Ordering::Relaxed);
            return false;
        };

        let start_index = usize::from(PROGRAM_START_INDEX);

        let mut heap = self.heap.lock().unwrap();

        if start_index + program.len() > heap.len() {
            eprintln!("Error: Program {program_path} is too large to fit in the heap.");
            self.active.store(false, Ordering::Relaxed);
            return false;
        }

        heap[start_index..start_index + program.len()].copy_from_slice(&program);

        return true;
    }

    pub fn write_byte(&self, val: u8, addr: u16) {
        self.heap.lock().unwrap()[usize::from(addr)] = val;
    }

    pub fn write_bytes(&self, vals: &Vec<u8>, addr: u16) -> bool {
        let addr = usize::from(addr);
        let count = vals.len();

        if addr + count > HEAP_SIZE {
            if !self.config.allow_heap_overflow {
                eprintln!("Error: Heap overflowed while writing.");
                self.active.store(false, Ordering::Relaxed);
                return false;
            }

            let mut heap = self.heap.lock().unwrap();

            let count_pre_split = HEAP_SIZE - addr;
            let count_post_split = count - count_pre_split;

            heap[addr..].copy_from_slice(&vals[..count_pre_split]);
            heap[..count_post_split].copy_from_slice(&vals[count_pre_split..]);

            return true;
        }

        let mut heap = self.heap.lock().unwrap();
        heap[addr..addr + count].copy_from_slice(&vals);
        return true;
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        return self.heap.lock().unwrap()[usize::from(addr)];
    }

    pub fn read_bytes(&self, addr: u16, count: u16) -> Option<Vec<u8>> {
        let addr = usize::from(addr);
        let count = usize::from(count);

        if addr + count > HEAP_SIZE {
            if !self.config.allow_heap_overflow {
                eprintln!("Error: Heap overflowed while reading.");
                self.active.store(false, Ordering::Relaxed);
                return None;
            }

            let heap = self.heap.lock().unwrap();

            let count_pre_split = HEAP_SIZE - addr;
            let count_post_split = count - count_pre_split;

            let mut bytes: Vec<u8> = Vec::with_capacity(count);
            bytes.extend_from_slice(&heap[addr..]);
            bytes.extend_from_slice(&heap[..count_post_split]);

            return Some(bytes);
        }

        let heap = self.heap.lock().unwrap();
        return Some(heap[addr..addr + count].to_vec());
    }

    pub fn push_to_stack(&self, val: u16) -> bool {
        let mut stack_ptr = self.stack_ptr.lock().unwrap();

        if *stack_ptr == self.config.stack_size {
            if !self.config.allow_stack_overflow {
                eprintln!("Error: Stack overflowed while pushing.");
                self.active.store(false, Ordering::Relaxed);
                return false;
            }

            let mut stack = self.stack.lock().unwrap();
        }

        // return true;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_test_config() -> Config {
        Config {
            instructions_per_second: 0.0,
            use_new_shift_instruction: false,
            use_new_jump_instruction: false,
            set_flag_for_index_overflow: false,
            move_index_with_reads: false,
            allow_program_counter_overflow: false,
            use_true_randomness: false,
            fake_randomness_seed: 0,

            // GPU Settings:
            horizontal_resolution: 0,
            vertical_resolution: 0,
            wrap_pixels: false,
            render_occasion: String::from(""),
            render_frequency: 0.0,

            // RAM Settings:
            stack_size: 16,
            allow_stack_overflow: true,
            allow_heap_overflow: true,
            font_start_index_on_heap: 0,
            font_data: [0; 80],

            // Timer settings:
            delay_timer_decrement_rate: 0.0,
            sound_timer_decrement_rate: 0.0,

            // Input Settings:
            key_bindings: [String::from("").clone(); 16],
        }
    }

    #[test]
    fn test_read_write_byte_to_memory() {
        let config = Arc::new(config::generate_config().unwrap());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        let ideal_byte = 0x56;
        let addr = 0x789;
        ram.write_byte(ideal_byte, addr);

        let actual_byte = ram.read_byte(addr);

        assert_eq!(ideal_byte, actual_byte);
    }

    #[test]
    fn test_read_bytes_from_memory() {
        let config = Arc::new(config::generate_config().unwrap());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        let ideal_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let start_addr: u16 = 0x789;

        for i in 0..5 {
            ram.write_byte(ideal_bytes[usize::from(i)], start_addr + i);
        }

        let actual_bytes = ram.read_bytes(start_addr, 5).unwrap();

        assert_eq!(ideal_bytes, actual_bytes);
    }

    #[test]
    fn test_write_bytes_to_memory() {
        let config = Arc::new(config::generate_config().unwrap());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        let ideal_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let start_addr: u16 = 0x789;

        ram.write_bytes(&ideal_bytes, start_addr);

        let mut actual_bytes: Vec<u8> = Vec::new();

        for i in start_addr..start_addr + 5 {
            let byte = ram.read_byte(i);
            actual_bytes.push(byte);
        }

        assert_eq!(ideal_bytes, actual_bytes);
    }

    #[test]
    fn test_load_program_to_memory() {
        let program = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let program_path = String::from("test.txt");
        fs::write(&program_path, &program).unwrap();

        let config = Arc::new(config::generate_config().unwrap());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        ram.load_program(&program_path);

        let ideal_bytes = vec![0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00];
        let actual_bytes = ram.read_bytes(ram::PROGRAM_START_INDEX - 1, 7).unwrap();

        assert_eq!(ideal_bytes, actual_bytes);
    }

    #[test]
    fn test_read_memory_with_overflow() {}
}
