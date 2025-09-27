use crate::config::Config;
use crate::emulib::Limiter;
use crate::instructions::{InstructionFunction, Opcode, get_instruction_function};
use crate::ram::{HEAP_SIZE, PROGRAM_START_INDEX, RAM};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct CPU {
    pub active: Arc<AtomicBool>,
    pub config: Arc<Config>,
    pub ram: Arc<RAM>,
    pub pc: Mutex<u16>,
    pub index: Mutex<u16>,
    pub v: Mutex<[u8; 16]>,
}

impl CPU {
    pub fn new(active: Arc<AtomicBool>, config: Arc<Config>, ram: Arc<RAM>) -> Self {
        Self {
            active,
            config,
            ram,
            pc: Mutex::new(PROGRAM_START_INDEX),
            index: Mutex::new(0),
            v: Mutex::new([0; 16]),
        }
    }

    pub fn run(&self) {
        let mut limiter = Limiter::new(self.config.instructions_per_second, true);

        while self.active.load(Ordering::Relaxed) {
            limiter.wait_if_early();

            let Some(instruction) = self.fetch_instruction() else {
                return;
            };

            let Some(function) = self.decode_instruction(&instruction) else {
                continue;
            };

            self.execute_instruction(&instruction, &function);
        }
    }

    fn fetch_instruction(&self) -> Option<Opcode> {
        let mut pc = self.pc.lock().unwrap();

        let Some(instruction) = self.ram.get_instruction(*pc) else {
            return None;
        };

        if *pc >= 0xFFE && !self.config.allow_program_counter_overflow {
            eprintln!("Error: Program counter overflowed.");
            return None;
        }

        *pc = (*pc + 2) % 0x1000;

        return Some(Opcode::new_u16(instruction));
    }

    fn decode_instruction(&self, instruction: &Opcode) -> Option<InstructionFunction> {
        get_instruction_function(&instruction)
    }

    fn execute_instruction(&self, instruction: &Opcode, function: &InstructionFunction) {
        function(&self, &instruction);
    }
}
