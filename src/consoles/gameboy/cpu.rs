use super::target::Target;

extern crate libc;

#[allow(unused_imports)]
use super::instruction::{
    AFFECTED, CARRY_FLAG, HALF_CARRY_FLAG, NOT_AFFECTED, RESET, SET, SUB_FLAG, ZERO_FLAG,
};

#[allow(unused_imports)]
use super::registers::{CARRY_BIT_POS, HALF_CARRY_BIT_POS, SUB_BIT_POS, ZERO_BIT_POS};
use std::mem;

use super::registers::Flag;
use super::instruction::Instruction;
use super::memory::Memory;
use super::opcode::OpCode;
use super::registers::Registers;

use std::ptr;

macro_rules! log {
    ($a: expr) => {
        println!("{}", stringify!($a));
        $a;
    };
}

macro_rules! panic_or_print {
    ($a: expr) => {
       
        panic!($a);
    };

    ($a: expr, $b: expr) => {
       
        panic!($a, $b);
    };

    ($a: expr, $b: expr, $c: expr) => {
        
        panic!($a, $b, $c);
    };
}

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    memory: Memory,
    pc: u16,
    sp: u16,
    addr: u16,
    is_prefixed: bool,
    interrupts_enabled: bool,
    is_stopped: bool,
}

#[allow(dead_code, unused_assignments)]
impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            memory: Memory::new(),
            pc: 0x100,
            sp: 0xE000, // Check what value the stack pointer is initialised to
            addr: 0,
            is_prefixed: false,
            interrupts_enabled: true,
            is_stopped: false,
        }
    }

    pub fn power_up(&mut self) {
        println!("cpu.power_up()");
        self.registers.set_combined_register(Target::BC, 0x0013);
        self.registers.set_combined_register(Target::DE, 0x00D8);
        self.registers.set_combined_register(Target::HL, 0x014D);
        self.sp = 0xFFFE;
        log!(self.memory.write_byte(0xFF05, 0x00));
        log!(self.memory.write_byte(0xFF06, 0x00));
        log!(self.memory.write_byte(0xFF07, 0x00));
        log!(self.memory.write_byte(0xFF10, 0x80));
        log!(self.memory.write_byte(0xFF11, 0xBF));
        log!(self.memory.write_byte(0xFF12, 0xF3));
        log!(self.memory.write_byte(0xFF14, 0xBF));
        log!(self.memory.write_byte(0xFF16, 0x3F));
        log!(self.memory.write_byte(0xFF17, 0x00));
        log!(self.memory.write_byte(0xFF19, 0xBF));
        log!(self.memory.write_byte(0xFF1A, 0x7F));
        log!(self.memory.write_byte(0xFF1B, 0xFF));
        log!(self.memory.write_byte(0xFF1C, 0x9F));
        log!(self.memory.write_byte(0xFF1E, 0xBF));
        log!(self.memory.write_byte(0xFF20, 0xFF));
        log!(self.memory.write_byte(0xFF21, 0x00));
        log!(self.memory.write_byte(0xFF22, 0x00));
        log!(self.memory.write_byte(0xFF23, 0xBF));
        log!(self.memory.write_byte(0xFF24, 0x77));
        log!(self.memory.write_byte(0xFF25, 0xF3));
        log!(self.memory.write_byte(0xFF26, 0xF1)); // 0xF1 for gb / 0xF0 for sgb
        log!(self.memory.write_byte(0xFF40, 0x91));
        log!(self.memory.write_byte(0xFF42, 0x00));
        log!(self.memory.write_byte(0xFF43, 0x00));
        log!(self.memory.write_byte(0xFF45, 0x00));
        log!(self.memory.write_byte(0xFF47, 0xFC));
        log!(self.memory.write_byte(0xFF48, 0xFF));
        log!(self.memory.write_byte(0xFF49, 0xFF));
        log!(self.memory.write_byte(0xFF4A, 0x00));
        log!(self.memory.write_byte(0xFF4B, 0x00));
        log!(self.memory.write_byte(0xFFFF, 0x00));
    }

    pub fn load_program(&mut self, program: Vec<u8>) {
        program.iter().enumerate().for_each(|(i, x)| {
            self.memory.write_byte(i as u16, program[i]);
        });
    }

    pub fn print_memory(&self) {
        self.memory.print_memory_readable();
    }

    pub fn dump_memory(&self, mut buffer: &mut Vec<String>) {
        self.memory.dump_memory(&mut buffer);
    }

    fn set_flags(&mut self, instruction: &Instruction, old_value: u8, new_value: u8) {
        match instruction.flags[ZERO_FLAG] {
            RESET => {
                self.registers.set_flag(Flag::Zero, false);
            }
            SET => {
                self.registers.set_flag(Flag::Zero, true);
            }
            AFFECTED => {
                if new_value == 0 {
                    self.registers.set_flag(Flag::Zero, true);
                }
            }
            _ => {}
        }

        match instruction.flags[SUB_FLAG] {
            RESET => {
                self.registers.set_flag(Flag::Sub, false);
            }
            SET => {
                self.registers.set_flag(Flag::Sub, true);
            }
            AFFECTED => {
                // Check if this arm ever gets executed
            }
            _ => {}
        }

        match instruction.flags[HALF_CARRY_FLAG] {
            RESET => {
                self.registers.set_flag(Flag::HalfCarry, false);
            }
            SET => {
                self.registers.set_flag(Flag::HalfCarry, true);
            }
            AFFECTED => {
                if instruction.flags[SUB_FLAG] == AFFECTED || instruction.flags[SUB_FLAG] == SET {
                    self.registers
                        .set_flag(Flag::HalfCarry, old_value > 0b1111 && new_value < 0b10000);
                } else if instruction.flags[SUB_FLAG] == RESET {
                    self.registers
                        .set_flag(Flag::HalfCarry, old_value < 0b10000 && new_value > 0b1111);
                }
            }
            _ => {}
        }

        match instruction.flags[CARRY_FLAG] {
            RESET => {
                self.registers.set_flag(Flag::Carry, false);
            }
            SET => {
                self.registers.set_flag(Flag::Carry, true);
            }
            AFFECTED => {
                if instruction.flags[SUB_FLAG] == AFFECTED || instruction.flags[SUB_FLAG] == SET {
                    self.registers
                        .set_flag(Flag::Carry, old_value > 0b0 && old_value < new_value);
                } else if instruction.flags[SUB_FLAG] == RESET {
                    self.registers
                        .set_flag(Flag::Carry, old_value < 0b11111111 && old_value > new_value);
                }
            }
            _ => {}
        }
    }

    fn set_flags_16(&mut self, instruction: &Instruction, old_value: u16, new_value: u16) {
        match instruction.flags[ZERO_FLAG] {
            RESET => {
                self.registers.set_flag(Flag::Zero, false);
            }
            SET => {
                self.registers.set_flag(Flag::Zero, true);
            }
            AFFECTED => {
                if new_value == 0 {
                    self.registers.set_flag(Flag::Zero, true);
                }
            }
            _ => {}
        }

        match instruction.flags[SUB_FLAG] {
            RESET => {
                self.registers.set_flag(Flag::Sub, false);
            }
            SET => {
                self.registers.set_flag(Flag::Sub, true);
            }
            AFFECTED => {
                // Check if this arm ever gets executed
            }
            _ => {}
        }

        match instruction.flags[HALF_CARRY_FLAG] {
            RESET => {
                self.registers.set_flag(Flag::HalfCarry, false);
            }
            SET => {
                self.registers.set_flag(Flag::HalfCarry, true);
            }
            AFFECTED => {
                if instruction.flags[SUB_FLAG] == AFFECTED || instruction.flags[SUB_FLAG] == SET {
                    self.registers.set_flag(
                        Flag::HalfCarry,
                        old_value > 0b11111111 && new_value < 0b100000000,
                    );
                } else if instruction.flags[SUB_FLAG] == RESET {
                    self.registers.set_flag(
                        Flag::HalfCarry,
                        old_value < 0b100000000 && new_value > 0b11111111,
                    );
                }
            }
            _ => {}
        }

        match instruction.flags[CARRY_FLAG] {
            RESET => {
                self.registers.set_flag(Flag::Carry, false);
            }
            SET => {
                self.registers.set_flag(Flag::Carry, true);
            }
            AFFECTED => {
                if instruction.flags[SUB_FLAG] == AFFECTED || instruction.flags[SUB_FLAG] == SET {
                    self.registers
                        .set_flag(Flag::Carry, old_value > 0b0 && old_value < new_value);
                } else if instruction.flags[SUB_FLAG] == RESET {
                    self.registers.set_flag(
                        Flag::Carry,
                        old_value < 0b1111111111111111 && old_value > new_value,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn run(&mut self) {
        while self.tick() {}
    }

    pub fn get_reg_a(&self) -> u8 {
        self.registers.a
    }

    pub fn write_to_memory(&mut self, address: u16, byte: u8) {
        self.memory.write_byte(address, byte);
    }

    pub fn read_memory(&self, address: u16) -> &u8 {
        self.memory.read_byte(address)
    }

    pub fn zero_memory(&mut self) {
        for i in 0..0xFFFF {
            self.memory.write_byte(i, 0);
        }
    }

    pub fn set_memory_to_end_of_program(&mut self) {
        for i in 0..0xFFFF {
            self.memory.write_byte(
                i,
                Instruction::byte_from_opcode(OpCode::EndOfProgram).unwrap(),
            );
        }
    }

    pub fn tick(&mut self) -> bool {
        let instruction_byte = self.memory.read_byte(self.pc);

        if instruction_byte == *Instruction::byte_from_opcode(OpCode::EndOfProgram).unwrap() {
            return false;
        }

        let pc_increment: u16 =
            if let Some(instruction) = Instruction::fetch(instruction_byte, self.is_prefixed) {
                self.is_prefixed = false;
                self.execute(instruction)
            } else {
                panic!(
                    "Uknown instruction: 0x{:x} @ address {}",
                    instruction_byte, self.pc
                );
            };

        self.pc = pc_increment;
        true
    }

    #[allow(unused_variables)]
    pub fn execute(&mut self, instruction: &Instruction) -> u16 {
        let mut pc_increment = instruction.length as u16;
        let mut cycles = instruction.cycles;

        match instruction.opcode.to_owned() {
            OpCode::ADC(target) => {
                self.adc(target);
            }
            OpCode::ADD(target) => {
                self.add(target);
            }
            OpCode::ADD16(dst, src) => {
                self.add_16(dst, src);
            }
            OpCode::AND(target) => {
                self.and(target);
            }
            OpCode::BIT(bit, target) => {
                self.bit(bit, target);
            }
            OpCode::CALL(flag) => {
                if self.call(flag) {
                    pc_increment = 0;
                }
            }
            OpCode::CALL_UC => {
                if self.registers.get_flag(Flag::Zero) {
                    self.call(Flag::Zero);
                } else {
                    self.call(Flag::NotZero);
                }
                pc_increment = 0;
            }
            OpCode::CCF => {
                self.registers
                    .set_flag(Flag::Carry, !self.registers.get_flag(Flag::Carry));
            }
            OpCode::CP(target) => {
                self.cp(target);
            }
            OpCode::CPL => {
                self.cpl();
            }
            OpCode::DDA => {
                self.dda();
            }
            OpCode::DEC(target) => {
                self.dec(target);
            }
            OpCode::DEC16(target) => {
                self.dec_16(target);
            }
            OpCode::DisableInterrupt => {
                self.interrupts_enabled = false;
            }
            OpCode::EnableInterrupt => {
                self.interrupts_enabled = true;
            }
            OpCode::HALT => {}
            OpCode::INC(target) => {
                self.inc(target);
            }
            OpCode::INC16(target) => {
                self.inc_16(target);
            }
            OpCode::JUMP(flag) => {
                if self.jump_by_flag(flag) {
                    pc_increment = 0;
                } else {
                    cycles = instruction.optional_cycles;
                }
            }
            OpCode::JP => {
                self.jump();
            }
            OpCode::JR(flag) => {
                self.jr(flag);
            }
            OpCode::JP_HL => {
                self.jump_hl();
            }
            OpCode::JRUC => {
                if self.registers.get_flag(Flag::Zero) {
                    self.jr(Flag::Zero);
                } else {
                    self.jr(Flag::NotZero);
                }
            }
            OpCode::LD(dst, src) => {
                if self.registers.is_16bit_target(dst) || self.registers.is_16bit_target(src) {
                    self.load_16(dst, src);
                } else {
                    self.ld(dst, src);
                }
            }
            OpCode::NOP => {}
            OpCode::OR(target) => {
                self.or(target);
            }
            OpCode::POP(target) => {
                self.pop(target);
            }
            OpCode::PUSH(target) => {
                self.push(target);
            }
            OpCode::CB => {
                self.is_prefixed = true;
            }
            OpCode::RES(bit, target) => {
                self.res(bit, target);
            }
            OpCode::RET(flag) => {
                if self.ret(flag) {
                    pc_increment = 0;
                }
            }
            OpCode::RET_UC => {
                if self.registers.get_flag(Flag::Zero) {
                    self.ret(Flag::Zero);
                } else {
                    self.ret(Flag::NotZero);
                }
                pc_increment = 0;
            }
            OpCode::RETI => {
                if self.registers.get_flag(Flag::Zero) {
                    self.ret(Flag::Zero);
                } else {
                    self.ret(Flag::NotZero);
                }
                pc_increment = 0;
                self.interrupts_enabled = true;
            }
            OpCode::RL(target) => {
                self.rl(target);
            }
            OpCode::RLA => {
                self.rla();
            }
            OpCode::RLC(target) => {
                self.rlc(target);
            }
            OpCode::RLCA => {
                self.rlca();
            }
            OpCode::RR(target) => {
                self.rr(target);
            }
            OpCode::RRA => {
                self.rra();
            }
            OpCode::RRC(target) => {
                self.rrc(target);
            }
            OpCode::RRCA => {
                self.rrca();
            }
            OpCode::RST(address) => {
                self.rst(address);
                pc_increment = 0;
            }
            OpCode::SBC(target) => {
                self.sbc(target);
            }
            OpCode::SCF => {
                self.registers.set_flag(Flag::Carry, true);
            }
            OpCode::SET(bit, target) => {
                self.set(bit, target);
            }
            OpCode::SUB(target) => {
                self.sub(target);
            }
            OpCode::STOP => {
                self.is_stopped = true;
            }
            OpCode::SLA(target) => {
                self.sla(target);
            }
            OpCode::SRL(target) => {
                self.srl(target);
            }
            OpCode::SRA(target) => {
                self.sra(target);
            }
            OpCode::SWAP(target) => {
                self.swap(target);
            }
            OpCode::XOR(target) => {
                self.xor(target);
            }
            OpCode::STORE(dst, src) => {
                self.store(dst, src);
            }
            _ => {
                panic!("Unimplemented");
            }
        }

        self.pc.wrapping_sub(pc_increment)
    }

    fn adc(&mut self, src: Target) {
        let v =  if src.is_16bit() {
            let address = self.registers.get_address(src);
            self.memory.read_byte(address)
        } else {
            self.registers.get_value(src)
        };
        self.registers.get_flag(Flag::Carry);
        self.registers.a += self.registers.get_value(src);
    }

    fn add(&mut self, src: Target) {
        let v =  if src.is_16bit() {
            let address = self.registers.get_address(src);
            self.memory.read_byte(address)
        } else {
            self.registers.get_value(src)
        };
        self.add_d8(v)
    }

    fn add_d8(&mut self, v: u8) {
       self.registers.a += v
    }

    fn add_16(&mut self, dst: Target, src: Target) {}

    fn and(&mut self, src: Target) {}

    fn bit(&mut self, bit_pos: u8, reg: Target) {}

    fn call(&mut self, flag: Flag) -> bool {

        // No flags affected
        false
    }

    fn cp(&mut self, reg: Target) {}

    fn cpl(&mut self) {}

    fn dda(&mut self) {}

    fn dec(&mut self, target: Target) {}

    fn dec_16(&mut self, target: Target) {}

    fn inc(&mut self, target: Target) {}

    fn inc_16(&mut self, target: Target) {}

    fn jump(&mut self) {}

    fn jump_by_flag(&mut self, flag: Flag) -> bool {
        false
    }

    fn jr(&mut self, flag: Flag) -> bool {
        false
    }

    fn jump_hl(&mut self) {}

    pub fn ld_d8(&mut self, dst: Target, value: u8) {
        self.registers.set_register(dst, value);
    }

    pub fn ld(&mut self, dst: Target, src: Target) {
        self.ld_d8(dst, self.registers.get_value(src))
    }

    pub fn load_16(&mut self, dst: Target, src: Target) {}

    fn or(&mut self, src: Target) {}

    fn pop(&mut self, target: Target) {}

    fn push(&mut self, target: Target) {}

    fn res(&mut self, bit: u8, reg: Target) {}

    fn ret(&mut self, flag: Flag) -> bool {
        false
    }

    fn rla(&mut self) {}

    fn rlca(&mut self) {}

    fn rra(&mut self) {}

    fn rrca(&mut self) {}

    fn rl(&mut self, reg: Target) {}

    fn rlc(&mut self, reg: Target) {}

    fn rr(&mut self, reg: Target) {}

    fn rrc(&mut self, reg: Target) {}

    fn rst(&mut self, address: u16) {}

    fn sbc(&mut self, reg: Target) {}

    fn set(&mut self, bit: u8, reg: Target) {}

    fn sl(&mut self, reg: Target) {}

    fn sla(&mut self, reg: Target) {}

    fn srl(&mut self, reg: Target) {}

    fn sra(&mut self, reg: Target) {}

    fn sub(&mut self, src: Target) {
        let v = self.registers.get_address(src);
        self.registers.a = self.registers.a as u16 - v
    }

    fn swap(&mut self, reg: Target) {}

    fn xor(&mut self, src: Target) {}

    fn store(&mut self, dst: Target, src: Target) {}

    fn store_16(&mut self, dst: Target, src: Target) {}

    pub fn set_a(&mut self, value: u8) {}

    pub fn set_b(&mut self, value: u8) {}

    pub fn print_registers(&self) {}

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn get_memory(&mut self) -> &mut Memory {
        &mut self.memory
    }

    pub fn reset_registers(&mut self) {
        self.pc = 0x100;
        self.sp = 0xE000;
        self.registers.reset();
    }
}

#[test]
fn test_adc() {
    let mut cpu = Cpu::new();

    cpu.registers.set_flag(Flag::Carry, true);
    cpu.registers.a = 254;
    cpu.registers.b = 1;
    cpu.adc(Target::B);

    assert!(cpu.registers.a == 0);
}

#[test]
fn test_add() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 0;
    cpu.registers.b = 5;
    cpu.add(Target::B);
    assert!(cpu.registers.a == 5);
    assert!(!cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(!cpu.registers.get_flag(Flag::HalfCarry));
    assert!(!cpu.registers.get_flag(Flag::Sub));

    cpu.registers.a = 255;
    cpu.registers.b = 1;
    cpu.add(Target::B);

    assert!(cpu.registers.a == 0);
    assert!(cpu.registers.get_flag(Flag::Zero));
    assert!(cpu.registers.get_flag(Flag::Carry));
    assert!(!cpu.registers.get_flag(Flag::HalfCarry));
    assert!(!cpu.registers.get_flag(Flag::Sub));
}

#[test]
fn test_add16() {
    let mut cpu = Cpu::new();

    cpu.registers.b = 1;
    cpu.registers.c = 0b10000000;

    cpu.add_16(Target::HL, Target::BC);

    eprintln!("h = {}\nl = {}", cpu.registers.h, cpu.registers.l);
    assert!(cpu.registers.h == 1 && cpu.registers.l == 0b10000000);
}

#[test]
fn test_and() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 3;
    cpu.registers.b = 5;
    cpu.and(Target::B);

    assert!(cpu.registers.a == 1);
}

#[test]
fn test_bit() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 0b10000000;
    cpu.bit(7, Target::A);

    assert!(!cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Sub));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(cpu.registers.get_flag(Flag::HalfCarry));

    cpu.registers.a = 0;
    cpu.bit(7, Target::A);

    assert!(cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Sub));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(cpu.registers.get_flag(Flag::HalfCarry));
}

#[test]
fn test_call_and_ret() {
    let mut cpu = Cpu::new();

    eprintln!("pc = {}", cpu.pc);
    let address1 = cpu.pc + 10;
    let address2 = cpu.pc;

    cpu.memory
        .write_byte(cpu.pc + 1, (address1 & 0b11111111) as u8);
    cpu.memory
        .write_byte(cpu.pc + 2, ((address1 & 0b1111111100000000) >> 8) as u8);
    assert!(cpu.call(Flag::NotZero));

    assert!(cpu.pc == address1);

    assert!(cpu.ret(Flag::NotZero));

    eprintln!("pc = {}", cpu.pc);
    assert!(cpu.pc == address2);
}

#[test]
fn test_cpl() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.cpl();
    assert!(cpu.registers.a == 0b11111110);
}

#[test]
fn test_dec() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.dec(Target::A);

    assert!(cpu.registers.a == 0);
    assert!(cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(!cpu.registers.get_flag(Flag::HalfCarry));
    assert!(cpu.registers.get_flag(Flag::Sub));
}

#[test]
fn test_dec_16() {
    let mut cpu = Cpu::new();

    cpu.registers.set_combined_register(Target::HL, 0b100000000);
    cpu.dec_16(Target::HL);

    assert!(cpu.registers.combined_register(Target::HL) == 0b11111111);
}

#[test]
fn test_inc() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 0;
    cpu.inc(Target::A);

    assert!(cpu.registers.a == 1);
    assert!(!cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(!cpu.registers.get_flag(Flag::HalfCarry));
    assert!(!cpu.registers.get_flag(Flag::Sub));
}

#[test]
fn test_inc_16() {
    let mut cpu = Cpu::new();

    cpu.sp = 0;
    cpu.inc_16(Target::SP);

    assert!(cpu.sp == 1);
}

#[test]
fn test_jump() {
    let mut cpu = Cpu::new();

    cpu.registers.set_combined_register(Target::HL, 100);
    cpu.jump();

    assert!(cpu.pc == 100);
}

#[test]
fn test_jump_hl() {
    let mut cpu = Cpu::new();

    cpu.registers
        .set_combined_register(Target::HL, 0b0000001010001000);

    cpu.jump_hl();

    assert!(cpu.pc == 0b0000001010001000);
}

#[test]
fn test_jump_by_flag() {
    let mut cpu = Cpu::new();

    cpu.zero_memory();

    let initial = cpu.pc as u8;

    cpu.registers.a = 5;
    cpu.registers.b = 5;
    cpu.memory.write_byte(cpu.pc + 1, initial + 255);
    cpu.memory.write_byte(cpu.pc + 2, initial + 0);
    cpu.sub(Target::B);
    cpu.jump_by_flag(Flag::Zero);

    assert!(cpu.pc as u8 == initial + 255);
    cpu.reset_registers();

    cpu.registers.b = 5;
    cpu.memory.write_byte(cpu.pc + 1, 0);
    cpu.memory.write_byte(cpu.pc + 2, 0);
    cpu.add(Target::B);
    cpu.jump_by_flag(Flag::NotZero);

    assert!(cpu.pc == 0);
    cpu.reset_registers();

    cpu.registers.a = 200;
    cpu.registers.b = 100;
    cpu.memory.write_byte(cpu.pc + 1, initial + 255);
    cpu.memory.write_byte(cpu.pc + 2, initial + 0);
    cpu.add(Target::B);

    cpu.jump_by_flag(Flag::Carry);

    assert!(cpu.pc as u8 == initial + 255);
    cpu.reset_registers();

    cpu.registers.b = 100;
    cpu.memory.write_byte(cpu.pc + 1, 0);
    cpu.memory.write_byte(cpu.pc + 2, 0);
    cpu.add(Target::B);

    cpu.jump_by_flag(Flag::NotCarry);

    assert!(cpu.pc == 0);
}

#[test]
fn test_jr() {
    let mut cpu = Cpu::new();

    let mut initial = cpu.pc as u8;
    cpu.write_to_memory(cpu.pc + 1, initial + 100);
    cpu.registers.a = 5;
    cpu.registers.b = 5;
    cpu.sub(Target::B);

    cpu.jr(Flag::Zero);

    assert!(cpu.pc as u8 == initial + 100);
    cpu.reset_registers();
    initial = cpu.pc as u8;

    cpu.write_to_memory(cpu.pc + 1, initial + 5);
    cpu.registers.b = 5;
    cpu.add(Target::B);

    cpu.jr(Flag::NotZero);

    assert!(cpu.pc as u8 == initial + 5);
    cpu.reset_registers();
    initial = cpu.pc as u8;

    cpu.memory.write_byte(cpu.pc + 1, initial + 100);
    cpu.registers.a = 255;
    cpu.registers.b = 20;
    cpu.add(Target::B);

    cpu.jr(Flag::Carry);

    assert!(cpu.pc as u8 == initial + 100);
    cpu.reset_registers();
    initial = cpu.pc as u8;

    cpu.memory.write_byte(cpu.pc + 1, initial + 105);
    cpu.registers.a = 100;
    cpu.registers.b = 20;
    cpu.add(Target::B);

    cpu.jr(Flag::NotCarry);

    assert!(cpu.pc as u8 == initial + 105);
}

#[test]
fn test_load() {
    let mut cpu = Cpu::new();
    cpu.write_to_memory(
        cpu.pc,
        Instruction::byte_from_opcode(OpCode::LD(Target::A, Target::D8)).unwrap(),
    );
    cpu.write_to_memory(cpu.pc + 1, 100);
    cpu.tick();
    cpu.tick();
    assert!(cpu.registers.a == 100);
}

#[test]
fn test_or() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 3;
    cpu.registers.b = 5;
    cpu.or(Target::B);

    assert!(cpu.registers.a == 7);
}

#[test]
fn test_push_and_pop() {
    let mut cpu = Cpu::new();

    cpu.registers
        .set_combined_register(Target::HL, 0b1000100000010001);

    cpu.push(Target::HL);

    assert!(*cpu.memory.read_byte(cpu.sp + 1) == 0b10001000);
    assert!(*cpu.memory.read_byte(cpu.sp + 2) == 0b00010001);

    cpu.pop(Target::BC);

    assert!(cpu.registers.combined_register(Target::BC) == 0b1000100000010001);
}

#[test]
fn test_sub() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 10;
    cpu.registers.b = 5;
    cpu.sub(Target::B);

    assert!(cpu.registers.a == 5);
    assert!(!cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(!cpu.registers.get_flag(Flag::HalfCarry));
    assert!(cpu.registers.get_flag(Flag::Sub));

    cpu.registers.a = 0;
    cpu.registers.b = 1;
    cpu.sub(Target::B);

    assert!(cpu.registers.a == 255);
    assert!(!cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(!cpu.registers.get_flag(Flag::HalfCarry));
    assert!(cpu.registers.get_flag(Flag::Sub));

    cpu.registers.a = 5;
    cpu.registers.b = 5;
    cpu.sub(Target::B);

    assert!(cpu.registers.a == 0);
    assert!(cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(!cpu.registers.get_flag(Flag::HalfCarry));
    assert!(cpu.registers.get_flag(Flag::Sub));
}

#[test]
fn test_flags() {
    let mut cpu = Cpu::new();

    let carry = cpu.registers.get_flag(Flag::Carry);
    cpu.registers.set_flag(Flag::Carry, !carry);
    assert!(carry != cpu.registers.get_flag(Flag::Carry));

    cpu.registers.set_flag(Flag::Carry, false);
    assert!(!cpu.registers.get_flag(Flag::Carry));

    cpu.registers.set_flag(Flag::Carry, true);
    assert!(cpu.registers.get_flag(Flag::Carry));
}

#[test]
fn test_res() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.res(0, Target::A);

    assert!(cpu.registers.a == 0);
}

#[test]
fn test_rl() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.rl(Target::A);

    assert!(cpu.registers.a == 2);

    cpu.registers.a = 0b10000000;
    cpu.rl(Target::A);

    assert!(cpu.registers.a == 1);
}

#[test]
fn test_rla() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 0b10000000;
    cpu.rla();

    assert!(cpu.registers.get_flag(Flag::Carry));
    assert!(cpu.registers.a == 0);

    cpu.rla();

    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(cpu.registers.a == 1);
}

#[test]
fn test_rlc() {
    let mut cpu = Cpu::new();

    cpu.registers.set_flag(Flag::Carry, true);
    cpu.registers.a = 1;
    cpu.rlc(Target::A);

    assert!(cpu.registers.a == 2);
    assert!(!cpu.registers.get_flag(Flag::Carry));

    cpu.registers.a = 0b10000000;
    cpu.rlc(Target::A);

    assert!(cpu.registers.a == 1);
    assert!(cpu.registers.get_flag(Flag::Carry));
}

#[test]
fn test_rr() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.rr(Target::A);

    assert!(cpu.registers.a == 0b10000000);

    cpu.registers.a = 2;
    cpu.rr(Target::A);

    assert!(cpu.registers.a == 1);
}

#[test]
fn test_rra() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.rra();

    assert!(cpu.registers.get_flag(Flag::Carry));
    assert!(cpu.registers.a == 0);

    cpu.rra();

    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(cpu.registers.a == 0b10000000);
}

#[test]
fn test_rrc() {
    let mut cpu = Cpu::new();

    cpu.registers.set_flag(Flag::Carry, true);
    cpu.registers.a = 1;
    cpu.rrc(Target::A);

    assert!(cpu.registers.a == 0b10000000);
    assert!(cpu.registers.get_flag(Flag::Carry));

    cpu.rrc(Target::A);

    assert!(cpu.registers.a == 0b01000000);
    assert!(!cpu.registers.get_flag(Flag::Carry));
}

#[test]
fn test_rrca() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.rrca();

    assert!(cpu.registers.get_flag(Flag::Carry));
    assert!(cpu.registers.a == (1 << ZERO_BIT_POS));
}

#[test]
fn test_sbc() {
    let mut cpu = Cpu::new();

    cpu.registers.set_flag(Flag::Carry, true);
    cpu.registers.a = 1;
    cpu.registers.b = 1;
    cpu.sbc(Target::B);

    assert!(cpu.registers.a == 0xFF);
}

#[test]
fn test_set() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 0;
    cpu.set(7, Target::A);

    assert!(cpu.registers.a == 128);

    cpu.registers.a = cpu.registers.a | 1;
    assert!(cpu.registers.a == 129);
}

#[test]
fn test_sl() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.sl(Target::A);

    assert!(cpu.registers.a == 2);

    cpu.registers.a = 0b10000000;
    cpu.sl(Target::A);

    assert!(cpu.registers.a == 0);
}

#[test]
fn test_sla() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.sla(Target::A);

    assert!(cpu.registers.a == 2);
}

#[test]
fn test_sr() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 2;
    cpu.srl(Target::A);

    assert!(cpu.registers.a == 1);

    cpu.srl(Target::A);

    assert!(cpu.registers.a == 0);
}

#[test]
fn test_sra() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 0b10000000;
    cpu.sra(Target::A);

    assert!(cpu.registers.a == 0b11000000);
}

#[test]
fn test_swap() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 128 + 4;
    cpu.swap(Target::A);
    assert!(72 == cpu.registers.a);
}

#[test]
fn test_xor() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 3;
    cpu.registers.b = 5;
    cpu.xor(Target::B);

    assert!(cpu.registers.a == 6);
}
