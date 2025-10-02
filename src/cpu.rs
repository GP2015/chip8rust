use crate::config::CPUConfig;
use crate::emulib::Limiter;
use crate::instructions::{InstructionFunction, Opcode, get_instruction_function};
use crate::ram::{PROGRAM_START_ADDRESS, RAM};
use crate::timer::{DelayTimer, SoundTimer};
use fastrand;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct CPU {
    pub active: Arc<AtomicBool>,
    pub config: CPUConfig,
    pub ram: Arc<RAM>,
    pub delay_timer: Arc<DelayTimer>,
    pub sound_timer: Arc<SoundTimer>,
    pc: Mutex<u16>,
    index: Mutex<u16>,
    v: Mutex<[u8; 16]>,
}

impl CPU {
    pub fn try_new(
        active: Arc<AtomicBool>,
        config: CPUConfig,
        ram: Arc<RAM>,
        delay_timer: Arc<DelayTimer>,
        sound_timer: Arc<SoundTimer>,
    ) -> Option<Arc<Self>> {
        if config.instructions_per_second == 0.0 {
            eprintln!("Error: The CPU cannot run at 0 instructions per second.");
            active.store(false, Ordering::Relaxed);
            return None;
        }

        return Some(Arc::new(Self {
            active,
            config,
            ram,
            delay_timer,
            sound_timer,
            pc: Mutex::new(PROGRAM_START_ADDRESS),
            index: Mutex::new(0),
            v: Mutex::new([0; 16]),
        }));
    }

    #[cfg(test)]
    pub fn new_default_all_false(
        active: Arc<AtomicBool>,
        ram: Arc<RAM>,
        delay_timer: Arc<DelayTimer>,
        sound_timer: Arc<SoundTimer>,
    ) -> Arc<Self> {
        CPU::try_new(
            active,
            CPUConfig {
                instructions_per_second: 700.0,
                use_new_shift_instruction: false,
                use_new_jump_instruction: false,
                set_flag_for_index_overflow: false,
                move_index_with_reads: false,
                allow_program_counter_overflow: false,
                use_true_randomness: false,
                fake_randomness_seed: 0,
            },
            ram,
            delay_timer,
            sound_timer,
        )
        .unwrap()
    }

    #[cfg(test)]
    pub fn new_default_all_true(
        active: Arc<AtomicBool>,
        ram: Arc<RAM>,
        delay_timer: Arc<DelayTimer>,
        sound_timer: Arc<SoundTimer>,
    ) -> Arc<Self> {
        CPU::try_new(
            active,
            CPUConfig {
                instructions_per_second: 700.0,
                use_new_shift_instruction: true,
                use_new_jump_instruction: true,
                set_flag_for_index_overflow: true,
                move_index_with_reads: true,
                allow_program_counter_overflow: true,
                use_true_randomness: true,
                fake_randomness_seed: 0,
            },
            ram,
            delay_timer,
            sound_timer,
        )
        .unwrap()
    }

    pub fn run(&self) {
        if !self.config.use_true_randomness {
            fastrand::seed(self.config.fake_randomness_seed);
        }

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

        if *pc >= 0xFFE && !self.config.allow_program_counter_overflow {
            eprintln!("Error: Program counter overflowed.");
            self.active.store(false, Ordering::Relaxed);
            return None;
        }

        let Some(instruction_bytes) = self.ram.read_bytes(*pc, 2) else {
            return None;
        };

        *pc = (*pc + 2) % 0x1000;

        return Some(Opcode::from_u8s(instruction_bytes[0], instruction_bytes[1]));
    }

    fn decode_instruction(&self, instruction: &Opcode) -> Option<InstructionFunction> {
        get_instruction_function(&instruction)
    }

    fn execute_instruction(&self, instruction: &Opcode, function: &InstructionFunction) {
        function(&self, &instruction);
    }

    pub fn get_pc_ref(&self) -> MutexGuard<u16> {
        return self.pc.lock().unwrap();
    }

    pub fn get_pc(&self) -> u16 {
        return *self.pc.lock().unwrap();
    }

    pub fn set_pc(&self, value: u16) {
        *self.pc.lock().unwrap() = value;
    }

    pub fn increment_pc(&self) -> bool {
        let mut pc = self.pc.lock().unwrap();

        if *pc >= 0xFFE && !self.config.allow_program_counter_overflow {
            eprintln!("Error: Program counter overflowed.");
            self.active.store(false, Ordering::Relaxed);
            return false;
        }

        *pc = (*pc + 2) % 0x1000;
        return true;
    }

    pub fn get_index_reg_ref(&self) -> MutexGuard<u16> {
        return self.index.lock().unwrap();
    }

    pub fn get_index_reg(&self) -> u16 {
        return *self.index.lock().unwrap();
    }

    pub fn set_index_reg(&self, value: u16) {
        *self.index.lock().unwrap() = value;
    }

    pub fn get_v_regs_ref(&self) -> MutexGuard<[u8; 16]> {
        return self.v.lock().unwrap();
    }

    pub fn get_v_reg(&self, reg: u8) -> u8 {
        return self.v.lock().unwrap()[reg as usize];
    }

    pub fn get_v_reg_xy(&self, x: u8, y: u8) -> (u8, u8) {
        let v = self.v.lock().unwrap();
        return (v[x as usize], v[y as usize]);
    }

    pub fn get_v_reg_range(&self, range: Range<usize>) -> Vec<u8> {
        return self.v.lock().unwrap()[range].to_vec();
    }

    pub fn set_v_reg(&self, reg: u8, val: u8) {
        self.v.lock().unwrap()[(reg & 0x0F) as usize] = val;
    }

    pub fn set_v_reg_range(&self, reg: u8, vals: Vec<u8>) {
        let reg = reg as usize;
        self.v.lock().unwrap()[reg..reg + vals.len()].copy_from_slice(&vals);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum ConfigType {
        CONSERVATIVE,
        LIBERAL,
    }

    fn create_objects(cfg_type: ConfigType) -> (Arc<CPU>, Arc<AtomicBool>) {
        let active = Arc::new(AtomicBool::new(true));
        let ram = RAM::new_default_conservative(active.clone());
        let delay_timer = DelayTimer::new_default(active.clone());
        let sound_timer = SoundTimer::new_default(active.clone());
        let cpu = match cfg_type {
            ConfigType::CONSERVATIVE => {
                CPU::new_default_all_false(active.clone(), ram, delay_timer, sound_timer)
            }
            ConfigType::LIBERAL => {
                CPU::new_default_all_true(active.clone(), ram, delay_timer, sound_timer)
            }
        };

        return (cpu, active);
    }

    #[test]
    fn test_program_counter_increment() {
        let (cpu, active) = create_objects(ConfigType::CONSERVATIVE);

        let old_val = *cpu.pc.lock().unwrap();

        for _ in 0..5 {
            assert!(cpu.increment_pc());
        }

        assert_eq!(old_val + 10, *cpu.pc.lock().unwrap());
        assert!(active.load(Ordering::Relaxed));
    }

    #[test]
    fn test_program_counter_successful_overflow() {
        let (cpu, active) = create_objects(ConfigType::LIBERAL);

        for _ in 0..((0x1000 - PROGRAM_START_ADDRESS) / 2) {
            assert!(cpu.increment_pc());
        }

        assert_eq!(0x000, *cpu.pc.lock().unwrap());
        assert!(active.load(Ordering::Relaxed));
    }

    #[test]
    fn test_program_counter_failed_overflow() {
        let (cpu, active) = create_objects(ConfigType::CONSERVATIVE);

        for _ in 0..((0xFFF - PROGRAM_START_ADDRESS) / 2) {
            assert!(cpu.increment_pc());
        }

        assert!(!cpu.increment_pc());
        assert!(!active.load(Ordering::Relaxed));
    }
}
