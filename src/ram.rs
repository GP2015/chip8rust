use crate::config::RAMConfig;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const PROGRAM_START_INDEX: u16 = 0x200;
pub const HEAP_SIZE: usize = 0x1000;

pub struct RAM {
    active: Arc<AtomicBool>,
    config: Arc<RAMConfig>,
    heap: Mutex<[u8; HEAP_SIZE]>,
    stack: Mutex<Vec<u16>>,
    stack_ptr: Mutex<usize>,
}

impl RAM {
    pub fn new(active: Arc<AtomicBool>, config: Arc<RAMConfig>) -> Self {
        Self {
            active,
            heap: Mutex::new([0; HEAP_SIZE]),
            stack: Mutex::new(vec![0; config.stack_size]),
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
            stack[0] = val;
            *stack_ptr = 1;

            return true;
        }

        let mut stack = self.stack.lock().unwrap();
        stack[*stack_ptr] = val;
        *stack_ptr += 1;

        return true;
    }

    pub fn pop_from_stack(&self) -> Option<u16> {
        let mut stack_ptr = self.stack_ptr.lock().unwrap();

        if *stack_ptr == 0 {
            if !self.config.allow_stack_overflow {
                eprintln!("Error: Stack overflowed while popping.");
                self.active.store(false, Ordering::Relaxed);
                return None;
            }

            let stack = self.stack.lock().unwrap();
            *stack_ptr = self.config.stack_size - 1;
            return Some(stack[*stack_ptr]);
        }

        let stack = self.stack.lock().unwrap();
        *stack_ptr -= 1;
        return Some(stack[*stack_ptr]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config_liberal() -> RAMConfig {
        RAMConfig {
            stack_size: 16,
            allow_stack_overflow: true,
            allow_heap_overflow: true,
            font_start_index_on_heap: 0,
            font_data: [0; 80],
        }
    }

    fn test_config_conservative() -> RAMConfig {
        RAMConfig {
            stack_size: 16,
            allow_stack_overflow: false,
            allow_heap_overflow: false,
            font_start_index_on_heap: 0,
            font_data: [0; 80],
        }
    }

    #[test]
    fn test_read_write_byte_to_memory() {
        let config = Arc::new(test_config_conservative());
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
        let config = Arc::new(test_config_conservative());
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
        let config = Arc::new(test_config_conservative());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        let ideal_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let start_addr: u16 = 0x789;

        assert!(ram.write_bytes(&ideal_bytes, start_addr));

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
        let program_path = String::from("test_load_program_to_memory_temp_file.txt");
        fs::write(&program_path, &program).unwrap();

        let config = Arc::new(test_config_conservative());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        assert!(ram.load_program(&program_path));

        fs::remove_file(program_path).unwrap();

        let ideal_bytes = vec![0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00];
        let actual_bytes = ram.read_bytes(PROGRAM_START_INDEX - 1, 7).unwrap();

        assert_eq!(ideal_bytes, actual_bytes);
    }

    #[test]
    fn test_read_memory_with_successful_overflow() {
        let config = Arc::new(test_config_liberal());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        let ideal_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];

        assert!(ram.write_bytes(&ideal_bytes[..3].to_vec(), 0xFFD));
        assert!(ram.write_bytes(&ideal_bytes[3..].to_vec(), 0x000));

        let actual_bytes = ram.read_bytes(0xFFD, 5).unwrap();

        assert_eq!(ideal_bytes, actual_bytes);
    }

    #[test]
    fn test_read_memory_with_failed_overflow() {
        let config = Arc::new(test_config_conservative());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        let ideal_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];

        assert!(ram.write_bytes(&ideal_bytes[..3].to_vec(), 0xFFD));
        assert!(ram.write_bytes(&ideal_bytes[3..].to_vec(), 0x000));

        assert!(ram.read_bytes(0xFFD, 5).is_none());
    }

    #[test]
    fn test_write_memory_with_successful_overflow() {
        let config = Arc::new(test_config_liberal());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        let ideal_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];

        assert!(ram.write_bytes(&ideal_bytes, 0xFFD));

        let mut actual_bytes: Vec<u8> = Vec::with_capacity(5);
        actual_bytes.extend(ram.read_bytes(0xFFD, 3).unwrap());
        actual_bytes.extend(ram.read_bytes(0x000, 2).unwrap());

        assert_eq!(ideal_bytes, actual_bytes);
    }

    #[test]
    fn test_write_memory_with_failed_overflow() {
        let config = Arc::new(test_config_conservative());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        let ideal_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];

        assert!(!ram.write_bytes(&ideal_bytes, 0xFFD));
    }

    #[test]
    fn test_stack_push_pop() {
        let config = Arc::new(test_config_conservative());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        for i in 1..=5 {
            assert!(ram.push_to_stack(i));
        }

        for i in (1..=5).rev() {
            assert_eq!(i, ram.pop_from_stack().unwrap());
        }
    }

    #[test]
    fn test_stack_push_pop_with_successful_overflow() {
        let config = Arc::new(test_config_liberal());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        for i in 1..=20 {
            assert!(ram.push_to_stack(i));
        }

        for i in (5..=20).rev() {
            assert_eq!(i, ram.pop_from_stack().unwrap());
        }

        assert_eq!(20, ram.pop_from_stack().unwrap());
    }

    #[test]
    fn test_stack_push_with_failed_overflow() {
        let config = Arc::new(test_config_conservative());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        for i in 1..=16 {
            assert!(ram.push_to_stack(i));
        }

        assert!(!ram.push_to_stack(17));
    }

    #[test]
    fn test_stack_pop_with_failed_overflow() {
        let config = Arc::new(test_config_conservative());
        let active = Arc::new(AtomicBool::new(true));
        let ram = Arc::new(RAM::new(active, config));

        assert!(ram.pop_from_stack().is_none());
    }
}
