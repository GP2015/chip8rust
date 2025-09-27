use crate::cpu::CPU;

pub struct Opcode {
    full: u16,
}

impl Opcode {
    pub fn new_u16(full: u16) -> Self {
        Self { full }
    }

    pub fn new_u8s(high: u8, low: u8) -> Self {
        Self {
            full: (u16::from(high) << 8) | u16::from(low),
        }
    }

    pub fn get_full(&self) -> u16 {
        self.full
    }

    pub fn get_addr(&self) -> u16 {
        self.full & 0x0FFF
    }

    pub fn get_kk(&self) -> u8 {
        u8::try_from(self.full & 0x00FF).unwrap()
    }

    pub fn get_s(&self) -> u8 {
        u8::try_from((self.full & 0xF000) >> 12).unwrap()
    }

    pub fn get_x(&self) -> u8 {
        u8::try_from((self.full & 0x0F00) >> 8).unwrap()
    }

    pub fn get_y(&self) -> u8 {
        u8::try_from((self.full & 0x00F0) >> 4).unwrap()
    }

    pub fn get_n(&self) -> u8 {
        u8::try_from(self.full & 0x000F).unwrap()
    }
}

pub type InstructionFunction = fn(&CPU, &Opcode);

pub fn get_instruction_function(op: &Opcode) -> Option<InstructionFunction> {
    match op.get_s() {
        0x0 => match op.get_addr() {
            0x0E0 => Some(i_00E0_CLS),
            0x0EE => Some(i_00EE_RET),
            _ => {
                // eprintln!("Error: Machine code routines are not supported.");
                None
            }
        },

        0x1 => Some(i_1nnn_JP_addr),
        0x2 => Some(i_2nnn_CALL_addr),
        0x3 => Some(i_3xkk_SE_Vx_byte),
        0x4 => Some(i_4xkk_SNE_Vx_byte),

        0x5 => match op.get_n() {
            0x0 => Some(i_5xy0_SE_Vx_Vy),
            _ => {
                invalid_instruction_called();
                None
            }
        },

        0x6 => Some(i_6xkk_LD_Vx_byte),
        0x7 => Some(i_7xkk_ADD_Vx_byte),

        0x8 => match op.get_n() {
            0x0 => Some(i_8xy0_LD_Vx_Vy),
            0x1 => Some(i_8xy1_OR_Vx_Vy),
            0x2 => Some(i_8xy2_AND_Vx_Vy),
            0x3 => Some(i_8xy3_XOR_Vx_Vy),
            0x4 => Some(i_8xy4_ADD_Vx_Vy),
            0x5 => Some(i_8xy5_SUB_Vx_Vy),
            0x6 => Some(i_8xy6_SHR_Vx),
            0x7 => Some(i_8xy7_SUBN_Vx_Vy),
            0xE => Some(i_8xyE_SHL_Vx),
            _ => {
                invalid_instruction_called();
                None
            }
        },

        0x9 => match op.get_n() {
            0x0 => Some(i_9xy0_SNE_Vx_Vy),
            _ => {
                invalid_instruction_called();
                None
            }
        },

        0xA => Some(i_Annn_LD_I_addr),
        0xB => Some(i_Bnnn_JP_V0_addr),
        0xC => Some(i_Cxkk_RND_Vx_byte),
        0xD => Some(i_Dxyn_DRW_Vx_Vy_nibble),

        0xE => match op.get_kk() {
            0x9E => Some(i_Ex9E_SKP_Vx),
            0xA1 => Some(i_ExA1_SKNP_Vx),
            _ => {
                invalid_instruction_called();
                None
            }
        },

        0xF => match op.get_kk() {
            0x07 => Some(i_Fx07_LD_Vx_DT),
            0x0A => Some(i_Fx0A_LD_Vx_K),
            0x15 => Some(i_Fx15_LD_DT_Vx),
            0x18 => Some(i_Fx18_LD_ST_Vx),
            0x1E => Some(i_Fx1E_ADD_I_Vx),
            0x29 => Some(i_Fx29_LD_F_Vx),
            0x33 => Some(i_Fx33_LD_B_Vx),
            0x55 => Some(i_Fx55_LD_I_Vx),
            0x65 => Some(i_Fx65_LD_Vx_I),
            _ => {
                invalid_instruction_called();
                None
            }
        },

        _ => panic!("op.get_s() should not be returning a byte > 0x0F"),
    }
}

fn invalid_instruction_called() {
    eprintln!("Error: Invalid instruction called.");
}

fn i_00E0_CLS(cpu: &CPU, op: &Opcode) {}
fn i_00EE_RET(cpu: &CPU, op: &Opcode) {}

fn i_1nnn_JP_addr(cpu: &CPU, op: &Opcode) {
    let mut pc = cpu.pc.lock().unwrap();
    *pc = op.get_addr();
}

fn i_2nnn_CALL_addr(cpu: &CPU, op: &Opcode) {}
fn i_3xkk_SE_Vx_byte(cpu: &CPU, op: &Opcode) {}
fn i_4xkk_SNE_Vx_byte(cpu: &CPU, op: &Opcode) {}
fn i_5xy0_SE_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_6xkk_LD_Vx_byte(cpu: &CPU, op: &Opcode) {}
fn i_7xkk_ADD_Vx_byte(cpu: &CPU, op: &Opcode) {}
fn i_8xy0_LD_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_8xy1_OR_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_8xy2_AND_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_8xy3_XOR_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_8xy4_ADD_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_8xy5_SUB_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_8xy6_SHR_Vx(cpu: &CPU, op: &Opcode) {}
fn i_8xy7_SUBN_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_8xyE_SHL_Vx(cpu: &CPU, op: &Opcode) {}
fn i_9xy0_SNE_Vx_Vy(cpu: &CPU, op: &Opcode) {}
fn i_Annn_LD_I_addr(cpu: &CPU, op: &Opcode) {}
fn i_Bnnn_JP_V0_addr(cpu: &CPU, op: &Opcode) {}
fn i_Cxkk_RND_Vx_byte(cpu: &CPU, op: &Opcode) {}
fn i_Dxyn_DRW_Vx_Vy_nibble(cpu: &CPU, op: &Opcode) {}
fn i_Ex9E_SKP_Vx(cpu: &CPU, op: &Opcode) {}
fn i_ExA1_SKNP_Vx(cpu: &CPU, op: &Opcode) {}
fn i_Fx07_LD_Vx_DT(cpu: &CPU, op: &Opcode) {}
fn i_Fx0A_LD_Vx_K(cpu: &CPU, op: &Opcode) {}
fn i_Fx15_LD_DT_Vx(cpu: &CPU, op: &Opcode) {}
fn i_Fx18_LD_ST_Vx(cpu: &CPU, op: &Opcode) {}
fn i_Fx1E_ADD_I_Vx(cpu: &CPU, op: &Opcode) {}
fn i_Fx29_LD_F_Vx(cpu: &CPU, op: &Opcode) {}
fn i_Fx33_LD_B_Vx(cpu: &CPU, op: &Opcode) {}
fn i_Fx55_LD_I_Vx(cpu: &CPU, op: &Opcode) {}
fn i_Fx65_LD_Vx_I(cpu: &CPU, op: &Opcode) {}
