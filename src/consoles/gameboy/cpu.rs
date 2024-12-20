use crate::consoles::bus::Bus;
use crate::consoles::readable::Readable;
use crate::consoles::writeable::Writeable;

use super::target::Target;
extern crate libc;

#[allow(unused_imports)]
use super::instruction::{
    AFFECTED, CARRY_FLAG, HALF_CARRY_FLAG, NOT_AFFECTED, RESET, SET, SUB_FLAG, ZERO_FLAG,
};

#[allow(unused_imports)]
use super::registers::{CARRY_BIT_POS, HALF_CARRY_BIT_POS, SUB_BIT_POS, ZERO_BIT_POS};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use super::instruction::Instruction;
use super::memory::Memory;
use super::opcode::OpCode;
use super::registers::Flag;
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
    bus: Bus,
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
        let mut bus = Bus::new();
        let memory = Rc::new(RefCell::new(Memory::new()));
        bus.connect_readable(memory.clone());
        bus.connect_writeable(memory);
        Cpu {
            registers: Registers::new(),
            bus,
            pc: 0x100,
            sp: 0xE000, // Check what value the stack pointer is initialised to
            addr: 0,
            is_prefixed: false,
            interrupts_enabled: true,
            is_stopped: false,
        }
    }

    pub fn power_up(&mut self) {
        // println!("cpu.power_up()");
        // self.registers.set_combined_register(Target::BC, 0x0013);
        // self.registers.set_combined_register(Target::DE, 0x00D8);
        // self.registers.set_combined_register(Target::HL, 0x014D);
        // self.sp = 0xFFFE;
        // log!(self.memory.write_byte(0xFF05, 0x00));
        // log!(self.memory.write_byte(0xFF06, 0x00));
        // log!(self.memory.write_byte(0xFF07, 0x00));
        // log!(self.memory.write_byte(0xFF10, 0x80));
        // log!(self.memory.write_byte(0xFF11, 0xBF));
        // log!(self.memory.write_byte(0xFF12, 0xF3));
        // log!(self.memory.write_byte(0xFF14, 0xBF));
        // log!(self.memory.write_byte(0xFF16, 0x3F));
        // log!(self.memory.write_byte(0xFF17, 0x00));
        // log!(self.memory.write_byte(0xFF19, 0xBF));
        // log!(self.memory.write_byte(0xFF1A, 0x7F));
        // log!(self.memory.write_byte(0xFF1B, 0xFF));
        // log!(self.memory.write_byte(0xFF1C, 0x9F));
        // log!(self.memory.write_byte(0xFF1E, 0xBF));
        // log!(self.memory.write_byte(0xFF20, 0xFF));
        // log!(self.memory.write_byte(0xFF21, 0x00));
        // log!(self.memory.write_byte(0xFF22, 0x00));
        // log!(self.memory.write_byte(0xFF23, 0xBF));
        // log!(self.memory.write_byte(0xFF24, 0x77));
        // log!(self.memory.write_byte(0xFF25, 0xF3));
        // log!(self.memory.write_byte(0xFF26, 0xF1)); // 0xF1 for gb / 0xF0 for sgb
        // log!(self.memory.write_byte(0xFF40, 0x91));
        // log!(self.memory.write_byte(0xFF42, 0x00));
        // log!(self.memory.write_byte(0xFF43, 0x00));
        // log!(self.memory.write_byte(0xFF45, 0x00));
        // log!(self.memory.write_byte(0xFF47, 0xFC));
        // log!(self.memory.write_byte(0xFF48, 0xFF));
        // log!(self.memory.write_byte(0xFF49, 0xFF));
        // log!(self.memory.write_byte(0xFF4A, 0x00));
        // log!(self.memory.write_byte(0xFF4B, 0x00));
        // log!(self.memory.write_byte(0xFFFF, 0x00));
    }

    pub fn load_program(&mut self, program: Vec<u8>) {
        for i in 0..program.len() {
            self.bus.write(i as u16, program[i]);
        }
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
                unimplemented!()
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
        self.bus.write(address, byte);
    }

    pub fn read_memory(&self, address: u16) -> u8 {
        self.bus.read(address).unwrap()
    }

    pub fn zero_memory(&mut self) {
        for i in 0..0xFFFF {
            self.bus.write(i, 0);
        }
    }

    pub fn set_memory_to_end_of_program(&mut self) {
        for i in 0..0xFFFF {
            self.bus.write(
                i,
                Instruction::byte_from_opcode(OpCode::EndOfProgram).unwrap(),
            );
        }
    }

    pub fn tick(&mut self) -> bool {
        let instruction_byte = self.bus.read(self.pc).unwrap();

        if instruction_byte == Instruction::byte_from_opcode(OpCode::EndOfProgram).unwrap() {
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
                    self.load(dst, src);
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
            OpCode::XOR(target) => self.xor(target),
            OpCode::STORE(dst, src) => self.store(dst, src),
            _ => {
                panic!("Unimplemented");
            }
        }

        self.pc.wrapping_sub(pc_increment)
    }

    fn adc(&mut self, reg: Target) {
        let v = match reg {
            Target::A => self.registers.a,
            Target::B => self.registers.b,
            Target::C => self.registers.c,
            Target::D => self.registers.d,
            Target::E => self.registers.e,
            Target::H => self.registers.h,
            Target::L => self.registers.l,
            Target::HL => self
                .bus
                .read(self.registers.combined_register(reg))
                .unwrap(),
            Target::D8 => self.bus.read(self.pc + 1).unwrap(),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::ADC(reg)));
            }
        };

        let carry = self.registers.filter_flag(Flag::Carry);
        // self.registers
        //     .set_flag(Flag::Zero, self.registers.a as u16 + v + carry == 256);
        // self.registers
        //     .set_flag(Flag::Carry, self.registers.a as u16 + v + carry > 255);
        // self.registers.set_flag(
        //     Flag::HalfCarry,
        //     self.registers.a < 0b10000 && self.registers.a as u16 + v + carry > 0b1111,
        // );
        // self.registers.set_flag(Flag::Sub, false);
        self.registers.a = self.registers.a.wrapping_add(v).wrapping_add(carry);

        self.set_flags(
            Instruction::from_opcode(OpCode::ADC(reg)).unwrap(),
            v,
            self.registers.a,
        );
    }

    fn add(&mut self, src: Target) {
        let v = match src {
            Target::A => self.registers.a,
            Target::B => self.registers.b,
            Target::C => self.registers.c,
            Target::D => self.registers.d,
            Target::E => self.registers.e,
            Target::F => self.registers.f,
            Target::L => self.registers.l,
            Target::H => self.registers.h,
            Target::HL => self
                .bus
                .read(self.registers.combined_register(Target::HL))
                .unwrap(),
            Target::D8 => self.bus.read(self.pc + 1).unwrap(),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::ADD(src)));
            }
        };

        // self.registers
        //     .set_flag(Flag::Carry, self.registers.a as i32 + v as i32 > 255);

        // self.registers.set_flag(
        //     Flag::HalfCarry,
        //     self.registers.a < 0b1111 && self.registers.a + v >= 0b1111,
        // );

        self.registers.a = self.registers.a.wrapping_add(v);
        // self.registers.set_flag(Flag::Zero, self.registers.a == 0);
        // self.registers.set_flag(Flag::Sub, false);

        self.set_flags(
            Instruction::from_opcode(OpCode::ADD(src)).unwrap(),
            v,
            self.registers.a,
        );
    }

    #[allow(unused_assignments)]
    fn add_16(&mut self, dst: Target, src: Target) {
        let v: u16 = match src {
            Target::HL => ((self.registers.h as u16) << 8) + self.registers.l as u16,
            Target::BC => ((self.registers.b as u16) << 8) + self.registers.c as u16,
            Target::DE => ((self.registers.d as u16) << 8) + self.registers.e as u16,
            Target::SP => self.sp,
            Target::R8 => self.bus.read(self.pc + 1).unwrap() as u16,
            _ => {
                panic_or_print!(
                    "Unimplemented {}",
                    format!("{:#?}", OpCode::ADD16(dst, src))
                );
            }
        };

        let (old, new) = match dst {
            Target::HL => {
                let old = self.registers.combined_register(dst);
                let new = old + v;
                self.registers.l = (new & 0xFF) as u8;
                self.registers.h = (new >> 8) as u8;
                (old, new)
            }
            Target::SP => {
                let old = self.sp;
                self.sp += v;
                let new = self.sp;
                (old, new)
            }
            _ => {
                panic_or_print!(
                    "Unimplemented {}",
                    format!("{:#?}", OpCode::ADD16(dst, src))
                );
            }
        };

        self.set_flags_16(
            Instruction::from_opcode(OpCode::ADD16(dst, src)).unwrap(),
            old,
            new,
        );
    }

    fn and(&mut self, src: Target) {
        let old = self.registers.a;
        match src {
            // Target::A => self.registers.a,
            Target::B => self.registers.a &= self.registers.b,
            Target::C => self.registers.a &= self.registers.c,
            Target::D => self.registers.a &= self.registers.d,
            Target::E => self.registers.a &= self.registers.e,
            Target::F => self.registers.a &= self.registers.f,
            Target::L => self.registers.a &= self.registers.l,
            Target::H => self.registers.a &= self.registers.h,
            Target::HL => {
                self.registers.a &= self
                    .bus
                    .read(self.registers.combined_register(src))
                    .unwrap()
            }
            Target::D8 => self.registers.a &= self.bus.read(self.pc + 1).unwrap(),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::AND(src)));
            }
        }
        // self.registers.set_flag(Flag::Zero, self.registers.a == 0);
        // self.registers.set_flag(Flag::Sub, false);
        // self.registers.set_flag(Flag::HalfCarry, true);
        // self.registers.set_flag(Flag::Carry, false);
        self.set_flags(
            Instruction::from_opcode(OpCode::AND(src)).unwrap(),
            old,
            self.registers.a,
        );
    }

    fn bit(&mut self, bit_pos: u8, reg: Target) {
        let v = match reg {
            Target::A => self.registers.a,
            Target::B => self.registers.b,
            Target::C => self.registers.c,
            Target::D => self.registers.d,
            Target::E => self.registers.e,
            Target::H => self.registers.h,
            Target::L => self.registers.l,
            // Target::HL => v = ((self.registers.h as u16) << 8) | self.registers.l as u16,
            _ => {
                panic_or_print!(
                    "Unimplemented {}",
                    format!("{:#?}", OpCode::BIT(bit_pos, reg))
                );
            }
        };
        let bit = v & (1 << bit_pos);

        // Carry not affected
        // self.registers.set_flag(Flag::Zero, bit == 0);
        // self.registers.set_flag(Flag::Sub, false);
        // self.registers.set_flag(Flag::HalfCarry, true);
        self.set_flags(
            Instruction::from_opcode(OpCode::BIT(bit_pos, reg)).unwrap(),
            v,
            bit,
        );
    }

    fn call(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            let _ = self
                .bus
                .write(self.sp - 1, ((self.pc & 0b1111111100000000) >> 8) as u8);
            let _ = self.bus.write(self.sp, (self.pc & 0b11111111) as u8);

            self.sp = self.sp.wrapping_sub(2);

            self.pc = ((self.bus.read(self.pc + 2).unwrap() as u16) << 8)
                | self.bus.read(self.pc + 1).unwrap() as u16;

            true
        } else {
            // No flags affected
            false
        }
    }

    fn cp(&mut self, reg: Target) {
        let v = match reg {
            Target::A => self.registers.a as u16,
            Target::B => self.registers.b as u16,
            Target::C => self.registers.c as u16,
            Target::D => self.registers.d as u16,
            Target::E => self.registers.e as u16,
            Target::H => self.registers.h as u16,
            Target::L => self.registers.l as u16,
            Target::HL => ((self.registers.h as u16) << 8) | self.registers.l as u16,
            Target::D8 => self.bus.read(self.pc + 1).unwrap() as u16,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::CP(reg)));
            }
        };

        let result = self.registers.a as u16 - v;
        // self.registers.set_flag(Flag::Zero, result == 0);
        // self.registers.set_flag(Flag::Sub, true);
        // self.registers.set_flag(
        //     Flag::HalfCarry,
        //     self.registers.a > 0b1111 && result < 0b10000,
        // );
        // self.registers
        //     .set_flag(Flag::Carry, (self.registers.a as u16) < v); {

        self.set_flags_16(
            Instruction::from_opcode(OpCode::CP(reg)).unwrap(),
            v,
            result,
        );
    }

    fn cpl(&mut self) {
        self.registers.a = !self.registers.a;

        self.set_flags(Instruction::from_opcode(OpCode::CPL).unwrap(), 0, 0);
        // self.registers.set_flag(Flag::Sub, true);
        // self.registers.set_flag(Flag::HalfCarry, true);
    }

    fn dda(&mut self) {
        let mut correction: u8 = 0;
        let carry_flag = self.registers.get_flag(Flag::HalfCarry);

        if carry_flag || (self.registers.a & 0x0F) > 9 {
            correction |= 0x06;
        }

        if carry_flag || self.registers.a > 0x9F {
            correction |= 0x60;
            self.registers.set_flag(Flag::Carry, true);
        }

        self.registers.a = (self.registers.a).wrapping_add(correction);
        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
        self.registers.set_flag(Flag::HalfCarry, false);
        // Do not call self.set_flags() for this instruction, since it is an edge case
    }

    fn dec(&mut self, target: Target) {
        let f = |old: u8, cpu: &mut Cpu| {
            let new = (old).wrapping_sub(1);
            cpu.set_flags(
                Instruction::from_opcode(OpCode::DEC(target)).unwrap(),
                old,
                new,
            );
            new
        };
        match target {
            Target::A => self.registers.a = f(self.registers.a, self),
            Target::B => self.registers.b = f(self.registers.b, self),
            Target::C => self.registers.c = f(self.registers.c, self),
            Target::D => self.registers.d = f(self.registers.d, self),
            Target::E => self.registers.e = f(self.registers.e, self),
            Target::F => self.registers.f = f(self.registers.f, self),
            Target::L => self.registers.l = f(self.registers.l, self),
            Target::H => self.registers.h = f(self.registers.h, self),
            Target::HL => {
                let mut v = self
                    .bus
                    .read(self.registers.combined_register(target))
                    .unwrap();
                // self.registers.set_flag(Flag::HalfCarry, v == 0b10000);
                let old = v;
                v = v.wrapping_sub(1);
                // self.registers.set_flag(Flag::Zero, v == 0);
                // self.registers.set_flag(Flag::Sub, true);
                self.bus
                    .write(self.registers.combined_register(target), v)
                    .unwrap();
                self.set_flags(
                    Instruction::from_opcode(OpCode::DEC(target)).unwrap(),
                    old,
                    v,
                );
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::DEC(target)));
            }
        };

        // self.registers.set_flag(Flag::HalfCarry, false);
        // self.registers.set_flag(Flag::Carry, false);
        // self.registers.set_flag(Flag::Sub, true);
    }

    fn dec_16(&mut self, target: Target) {
        if target == Target::SP {
            let old = self.sp;
            self.sp = self.sp.wrapping_sub(1);
            let new = self.sp;
            self.set_flags_16(
                Instruction::from_opcode(OpCode::DEC16(target)).unwrap(),
                old,
                new,
            );
        } else {
            let old = self.registers.combined_register(target);
            let new = old.wrapping_sub(1);
            self.registers.set_combined_register(target, new);
            self.set_flags_16(
                Instruction::from_opcode(OpCode::DEC16(target)).unwrap(),
                old,
                new,
            );
        }
    }

    fn inc(&mut self, target: Target) {
        let f = |old: u8, cpu: &mut Cpu| {
            let new = old.wrapping_add(1);
            cpu.set_flags(
                Instruction::from_opcode(OpCode::INC(target)).unwrap(),
                old,
                new,
            );
            new
        };
        match target {
            Target::A => self.registers.a = f(self.registers.a, self),
            Target::B => self.registers.b = f(self.registers.b, self),
            Target::C => self.registers.c = f(self.registers.c, self),
            Target::D => self.registers.d = f(self.registers.d, self),
            Target::E => self.registers.e = f(self.registers.e, self),
            Target::F => self.registers.f = f(self.registers.f, self),
            Target::L => self.registers.l = f(self.registers.l, self),
            Target::H => self.registers.h = f(self.registers.h, self),
            Target::HL => {
                let mut v = self
                    .bus
                    .read(self.registers.combined_register(target))
                    .unwrap();
                let old = v;
                // self.registers.set_flag(Flag::HalfCarry, v == 0b1111);
                v = v.wrapping_add(1);
                // self.registers.set_flag(Flag::Zero, v == 0);
                // self.registers.set_flag(Flag::Sub, false);
                self.bus
                    .write(self.registers.combined_register(target), v)
                    .unwrap();
                self.set_flags(
                    Instruction::from_opcode(OpCode::INC(target)).unwrap(),
                    old,
                    v,
                );
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::INC(target)));
            }
        }
    }

    fn inc_16(&mut self, target: Target) {
        if target == Target::SP {
            let old = self.sp;
            self.sp = self.sp.wrapping_add(1);
            self.set_flags_16(
                Instruction::from_opcode(OpCode::INC16(target)).unwrap(),
                old,
                self.sp,
            );
        } else {
            let old = self.registers.combined_register(target);
            let new = old.wrapping_add(1);
            self.registers.set_combined_register(target, new);
            self.set_flags_16(
                Instruction::from_opcode(OpCode::INC16(target)).unwrap(),
                old,
                new,
            );
        }
    }

    fn jump(&mut self) {
        self.pc = self.registers.combined_register(Target::HL);
    }

    fn jump_by_flag(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = ((self.bus.read(self.pc + 2).unwrap() as u16) << 8)
                | self.bus.read(self.pc + 1).unwrap() as u16;
            true
        } else {
            false
        }
    }

    fn jr(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = self.pc + self.bus.read(self.pc + 1).unwrap() as u16;
            true
        } else {
            false
        }
    }

    fn jump_hl(&mut self) {
        self.pc = self.registers.combined_register(Target::HL);
    }

    #[allow(unused_assignments)]
    pub fn load(&mut self, dst: Target, src: Target) {
        let v = match src {
            Target::A => self.registers.a,
            Target::B => self.registers.b,
            Target::C => self.registers.c,
            Target::D => self.registers.d,
            Target::E => self.registers.e,
            Target::F => self.registers.f,
            Target::L => self.registers.l,
            Target::H => self.registers.h,
            Target::HL | Target::BC | Target::DE => self
                .bus
                .read(self.registers.combined_register(src))
                .unwrap(),
            Target::A8 => self
                .bus
                .read(0xFF00 + self.bus.read(self.pc + 1).unwrap() as u16)
                .unwrap(),
            Target::A16 => self
                .bus
                .read(
                    (self.bus.read(self.pc + 1).unwrap() as u16) << 8
                        | self.bus.read(self.pc + 2).unwrap() as u16,
                )
                .unwrap(),
            Target::R8 => self
                .bus
                .read(self.pc + self.bus.read(self.pc + 1).unwrap() as u16)
                .unwrap(),
            Target::D8 => self.bus.read(self.pc + 1).unwrap(),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        };

        match dst {
            Target::A => self.registers.a = v,
            Target::B => self.registers.b = v,
            Target::C => self.registers.c = v,
            Target::D => self.registers.d = v,
            Target::E => self.registers.e = v,
            Target::F => self.registers.f = v,
            Target::L => self.registers.l = v,
            Target::H => self.registers.h = v,
            Target::HL | Target::BC | Target::DE => {
                let _ = self.bus.write(self.registers.combined_register(dst), v);
            }
            Target::A8 => {
                let _ = self
                    .bus
                    .write(0xFF00 + self.bus.read(self.pc + 1).unwrap() as u16, v);
            }
            Target::A16 => {
                let _ = self.bus.write(
                    (self.bus.read(self.pc + 1).unwrap() as u16) << 8
                        | self.bus.read(self.pc + 2).unwrap() as u16,
                    v,
                );
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        };
    }

    pub fn load_16(&mut self, dst: Target, src: Target) {
        let v = match src {
            Target::A => self.registers.a as u16,
            Target::B => self.registers.b as u16,
            Target::C => self.registers.c as u16,
            Target::D => self.registers.d as u16,
            Target::E => self.registers.e as u16,
            Target::H => self.registers.h as u16,
            Target::L => self.registers.l as u16,
            Target::HL | Target::BC | Target::DE => self
                .bus
                .read(self.registers.combined_register(src))
                .unwrap() as u16,
            Target::A8 => self
                .bus
                .read(0xFF00 + self.bus.read(self.pc + 1).unwrap() as u16)
                .unwrap() as u16,
            Target::A16 => self
                .bus
                .read(
                    ((self.bus.read(self.pc + 1).unwrap() as u16) << 8)
                        + self.bus.read(self.pc + 2).unwrap() as u16,
                )
                .unwrap() as u16,
            Target::D8 => self.bus.read(self.pc + 1).unwrap() as u16,
            Target::D16 => {
                ((self.bus.read(self.pc + 1).unwrap() as u16) << 8)
                    + (self.bus.read(self.pc + 2).unwrap() as u16)
            }
            Target::SP => self.sp,
            Target::SP_R8 => self
                .bus
                .read(self.sp + self.bus.read(self.pc + 1).unwrap() as u16)
                .unwrap() as u16,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        };

        match dst {
            Target::HL | Target::BC | Target::DE => {
                let _ = self.bus.write_16(
                    self.bus
                        .read(self.registers.combined_register(dst))
                        .unwrap() as u16,
                    v,
                );
            }
            Target::A8 => {
                let _ = self
                    .bus
                    .write_16(0xFF00 + self.bus.read(self.pc + 1).unwrap() as u16, v);
            }
            Target::A16 => {
                let _ = self.bus.write_16(
                    ((self.bus.read(self.pc + 1).unwrap() as u16) << 8)
                        + self.bus.read(self.pc + 2).unwrap() as u16,
                    v,
                );
            }
            Target::SP => self.sp = v,
            Target::SP_R8 => {
                let _ = self
                    .bus
                    .write_16(self.sp + self.bus.read(self.pc + 1).unwrap() as u16, v);
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        }
    }

    fn or(&mut self, src: Target) {
        let old = self.registers.a;
        match src {
            Target::B => self.registers.a |= self.registers.b,
            Target::C => self.registers.a |= self.registers.c,
            Target::D => self.registers.a |= self.registers.d,
            Target::E => self.registers.a |= self.registers.e,
            Target::F => self.registers.a |= self.registers.f,
            Target::L => self.registers.a |= self.registers.l,
            Target::H => self.registers.a |= self.registers.h,
            Target::HL => {
                self.registers.a |= self
                    .bus
                    .read(self.registers.combined_register(src))
                    .unwrap()
            }
            Target::D8 => self.registers.a |= self.bus.read(self.pc + 1).unwrap(),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::OR(src)));
            }
        }

        self.set_flags(
            Instruction::from_opcode(OpCode::OR(src)).unwrap(),
            old,
            self.registers.a,
        );
    }

    fn pop(&mut self, target: Target) {
        let v = ((self.bus.read(self.sp + 1).unwrap() as u16) << 8)
            | self.bus.read(self.sp + 2).unwrap() as u16;

        self.sp = self.sp.wrapping_add(2);

        match target {
            Target::AF | Target::BC | Target::DE | Target::HL => {
                self.registers.set_combined_register(target, v)
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::POP(target)));
            }
        }
    }

    fn push(&mut self, target: Target) {
        let v = match target {
            Target::AF | Target::BC | Target::DE | Target::HL => {
                self.registers.combined_register(target)
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::PUSH(target)));
            }
        };

        let _ = self.bus.write(self.sp, (v & 0b11111111) as u8);
        let _ = self
            .bus
            .write(self.sp - 1, ((v & 0b1111111100000000) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(2);
    }

    fn res(&mut self, bit: u8, reg: Target) {
        let mask = !(1 << bit);
        match reg {
            Target::A => self.registers.a &= mask,
            Target::B => self.registers.b &= mask,
            Target::C => self.registers.c &= mask,
            Target::D => self.registers.d &= mask,
            Target::E => self.registers.e &= mask,
            Target::H => self.registers.h &= mask,
            Target::L => self.registers.l &= mask,
            // Target::HL => v = &mut self.registers.a,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RES(bit, reg)));
            }
        }
    }

    fn ret(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = ((self.bus.read(self.sp + 1).unwrap() as u16) << 8)
                | (self.bus.read(self.sp + 2).unwrap() as u16);
            self.sp = self.sp.wrapping_add(2);

            true
        } else {
            false
        }
    }

    fn rla(&mut self) {
        let msb = (self.registers.a & (1 << ZERO_BIT_POS)) >> ZERO_BIT_POS;
        let carry = self.registers.filter_flag(Flag::Carry);
        self.registers.a = self.registers.a << 1 | carry;
        self.registers.set_flag(Flag::Carry, msb != 0);
        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
        // Do not call self.set_flags()
    }

    fn rlca(&mut self) {
        self.registers
            .set_flag(Flag::Carry, (self.registers.a & (1 << ZERO_BIT_POS)) != 0);
        self.registers.a = self.registers.a.rotate_left(1);

        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
        // Do not call self.set_flags()
    }

    fn rra(&mut self) {
        let lsb = self.registers.a & 1;
        let carry = self.registers.filter_flag(Flag::Carry) << ZERO_BIT_POS;
        self.registers.a = self.registers.a >> 1 | carry;
        self.registers.set_flag(Flag::Carry, lsb != 0);
        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
        // Do not call self.set_flags()
    }

    fn rrca(&mut self) {
        self.registers
            .set_flag(Flag::Carry, (self.registers.a & 1) != 0);
        self.registers.a = self.registers.a.rotate_right(1);

        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
        // Do not call self.set_flags()
    }

    fn rl(&mut self, reg: Target) {
        let (old, new) = match reg {
            Target::A => {
                let old = self.registers.a;
                self.registers.a = self.registers.a.rotate_left(1);
                (old, self.registers.a)
            }
            Target::B => {
                let old = self.registers.b;
                self.registers.b = self.registers.b.rotate_left(1);
                (old, self.registers.b)
            }
            Target::C => {
                let old = self.registers.c;
                self.registers.c = self.registers.c.rotate_left(1);
                (old, self.registers.c)
            }
            Target::D => {
                let old = self.registers.d;
                self.registers.d = self.registers.d.rotate_left(1);
                (old, self.registers.d)
            }
            Target::E => {
                let old = self.registers.e;
                self.registers.e = self.registers.e.rotate_left(1);
                (old, self.registers.e)
            }
            Target::F => {
                let old = self.registers.f;
                self.registers.f = self.registers.f.rotate_left(1);
                (old, self.registers.f)
            }
            Target::H => {
                let old = self.registers.h;
                self.registers.h = self.registers.h.rotate_left(1);
                (old, self.registers.h)
            }
            Target::L => {
                let old = self.registers.l;
                self.registers.l = self.registers.l.rotate_left(1);
                (old, self.registers.l)
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RL(reg)));
            }
        };

        self.set_flags(Instruction::from_opcode(OpCode::RL(reg)).unwrap(), old, new);
    }

    fn rlc(&mut self, reg: Target) {
        let f = |old: u8, cpu: &mut Cpu| {
            cpu.registers
                .set_flag(Flag::Carry, (old & (1 << ZERO_BIT_POS)) != 0);
            let new = old.rotate_left(1);
            cpu.registers.set_flag(Flag::Zero, new == 0);
            new
        };
        match reg {
            Target::A => self.registers.a = f(self.registers.a, self),
            Target::B => self.registers.b = f(self.registers.b, self),
            Target::C => self.registers.c = f(self.registers.c, self),
            Target::D => self.registers.d = f(self.registers.d, self),
            Target::E => self.registers.e = f(self.registers.e, self),
            Target::F => self.registers.f = f(self.registers.f, self),
            Target::H => self.registers.h = f(self.registers.h, self),
            Target::L => self.registers.l = f(self.registers.l, self),
            Target::HL => {
                let v = self.registers.combined_register(reg);
                self.registers.set_flag(Flag::Carry, (v & (1 << 15)) != 0);
                self.registers.set_combined_register(reg, v.rotate_left(1));
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RLC(reg)));
            }
        };
    }

    fn rr(&mut self, reg: Target) {
        let (old, new) = match reg {
            Target::A => {
                let old = self.registers.a;
                self.registers.a = self.registers.a.rotate_right(1);
                (old, self.registers.a)
            }
            Target::B => {
                let old = self.registers.b;
                self.registers.b = self.registers.b.rotate_right(1);
                (old, self.registers.b)
            }
            Target::C => {
                let old = self.registers.c;
                self.registers.c = self.registers.c.rotate_right(1);
                (old, self.registers.c)
            }
            Target::D => {
                let old = self.registers.d;
                self.registers.d = self.registers.d.rotate_right(1);
                (old, self.registers.d)
            }
            Target::E => {
                let old = self.registers.e;
                self.registers.e = self.registers.e.rotate_right(1);
                (old, self.registers.e)
            }
            Target::F => {
                let old = self.registers.f;
                self.registers.f = self.registers.f.rotate_right(1);
                (old, self.registers.f)
            }
            Target::H => {
                let old = self.registers.h;
                self.registers.h = self.registers.h.rotate_right(1);
                (old, self.registers.h)
            }
            Target::L => {
                let old = self.registers.l;
                self.registers.l = self.registers.l.rotate_right(1);
                (old, self.registers.l)
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RR(reg)));
            }
        };

        self.set_flags(Instruction::from_opcode(OpCode::RR(reg)).unwrap(), old, new);
    }

    #[allow(unused_assignments)]
    fn rrc(&mut self, reg: Target) {
        let f = |old: u8, cpu: &mut Cpu| {
            cpu.registers.set_flag(Flag::Carry, (old & 1) != 0);
            let new = old.rotate_right(1);
            cpu.registers.set_flag(Flag::Zero, new == 0);
            new
        };
        match reg {
            Target::A => self.registers.a = f(self.registers.a, self),
            Target::B => self.registers.b = f(self.registers.b, self),
            Target::C => self.registers.c = f(self.registers.c, self),
            Target::D => self.registers.d = f(self.registers.d, self),
            Target::E => self.registers.e = f(self.registers.e, self),
            Target::F => self.registers.f = f(self.registers.f, self),
            Target::H => self.registers.h = f(self.registers.h, self),
            Target::L => self.registers.l = f(self.registers.a, self),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RRC(reg)));
            }
        };
    }

    fn rst(&mut self, address: u16) {
        let _ = self
            .bus
            .write(self.sp - 1, ((self.pc & 0b1111111100000000) >> 8) as u8);
        let _ = self.bus.write(self.sp, (self.pc & 0b11111111) as u8);

        self.sp = self.sp.wrapping_sub(2);

        self.pc = address;
    }

    fn sbc(&mut self, reg: Target) {
        let v = match reg {
            Target::A => self.registers.a,
            Target::B => self.registers.b,
            Target::C => self.registers.c,
            Target::D => self.registers.d,
            Target::E => self.registers.e,
            Target::H => self.registers.h,
            Target::L => self.registers.l,
            Target::HL => self
                .bus
                .read(self.registers.combined_register(reg))
                .unwrap(),
            Target::D8 => self.bus.read(self.pc + 1).unwrap(),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SBC(reg)));
            }
        };

        let carry = self.registers.filter_flag(Flag::Carry);
        let old = self.registers.a;

        // self.registers.set_flag(Flag::Zero, self.registers.a == v);
        // self.registers.set_flag(Flag::Sub, true);
        // self.registers.set_flag(
        //     Flag::Carry,
        //     self.registers.a > 0b1111 && self.registers.a - v - carry < 0b10000,
        // );
        // self.registers
        //     .set_flag(Flag::Zero, self.registers.a.lt(&(v + carry)));

        self.registers.a = self.registers.a.wrapping_sub(v).wrapping_sub(carry) as u8;

        self.set_flags(
            Instruction::from_opcode(OpCode::SBC(reg)).unwrap(),
            old,
            self.registers.a,
        );
    }

    fn set(&mut self, bit: u8, reg: Target) {
        let mask = 1 << bit;
        match reg {
            Target::A => self.registers.a |= mask,
            Target::B => self.registers.b |= mask,
            Target::C => self.registers.c |= mask,
            Target::D => self.registers.d |= mask,
            Target::E => self.registers.e |= mask,
            Target::H => self.registers.h |= mask,
            Target::L => self.registers.l |= mask,
            // Target::HL => v = &mut self.registers.h,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SET(bit, reg)));
            }
        }
    }

    fn sl(&mut self, reg: Target) {
        match reg {
            Target::A => self.registers.a <<= 1,
            Target::B => self.registers.b <<= 1,
            Target::C => self.registers.c <<= 1,
            Target::D => self.registers.d <<= 1,
            Target::E => self.registers.e <<= 1,
            Target::F => self.registers.f <<= 1,
            Target::H => self.registers.h <<= 1,
            Target::L => self.registers.l <<= 1,
            _ => {
                panic_or_print!("Unimplemented SL, NOT SURE IF THIS INSTRUCTION EXISTS");
            }
        }
    }

    #[allow(unused_assignments)]
    fn sla(&mut self, reg: Target) {
        let f = |old: u8, cpu: &mut Cpu| {
            let bit = old & (1 << ZERO_BIT_POS);
            let new = old << 1;
            cpu.registers.set_flag(Flag::Carry, bit == 0);
            cpu.registers.set_flag(Flag::Zero, new == 0);
            cpu.registers.set_flag(Flag::HalfCarry, false);
            cpu.registers.set_flag(Flag::Sub, false);
            new
        };

        match reg {
            Target::A => self.registers.a = f(self.registers.a, self),
            Target::B => self.registers.b = f(self.registers.b, self),
            Target::C => self.registers.c = f(self.registers.c, self),
            Target::D => self.registers.d = f(self.registers.d, self),
            Target::E => self.registers.e = f(self.registers.e, self),
            Target::F => self.registers.f = f(self.registers.f, self),
            Target::H => self.registers.h = f(self.registers.h, self),
            Target::L => self.registers.l = f(self.registers.l, self),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SLA(reg)));
            }
        };
    }

    fn srl(&mut self, reg: Target) {
        let f = |old: u8, cpu: &mut Cpu| {
            let lsb = old & 1;
            let new = old >> 1;
            cpu.registers.set_flag(Flag::Zero, new == 0);
            cpu.registers.set_flag(Flag::Sub, false);
            cpu.registers.set_flag(Flag::Carry, lsb != 0);
            cpu.registers.set_flag(Flag::HalfCarry, false);
            new
        };

        match reg {
            Target::A => self.registers.a = f(self.registers.a, self),
            Target::B => self.registers.b = f(self.registers.b, self),
            Target::C => self.registers.c = f(self.registers.c, self),
            Target::D => self.registers.d = f(self.registers.d, self),
            Target::E => self.registers.e = f(self.registers.e, self),
            Target::F => self.registers.f = f(self.registers.f, self),
            Target::H => self.registers.h = f(self.registers.h, self),
            Target::L => self.registers.l = f(self.registers.l, self),
            Target::HL => {
                let mut v = self.registers.combined_register(reg);
                let lsb = v & 1;
                v >>= 1;
                self.registers.set_flag(Flag::Zero, v == 0);
                self.registers.set_flag(Flag::Sub, false);
                self.registers.set_flag(Flag::Carry, lsb != 0);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_combined_register(reg, v);
                return;
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SRL(reg)));
            }
        };
    }

    #[allow(unused_assignments)]
    fn sra(&mut self, reg: Target) {
        let msb = |v: u8| v & (1 << ZERO_BIT_POS);
        let f = |old: u8, cpu: &mut Cpu| {
            let lsb = old & 1;
            let new = (old >> 1) | (msb(old));
            cpu.registers.set_flag(Flag::Zero, new == 0);
            cpu.registers.set_flag(Flag::Sub, false);
            cpu.registers.set_flag(Flag::Carry, lsb != 0);
            cpu.registers.set_flag(Flag::HalfCarry, false);
            cpu.registers.a = new;
        };
        match reg {
            Target::A => f(self.registers.a, self),
            Target::B => f(self.registers.b, self),
            Target::C => f(self.registers.c, self),
            Target::D => f(self.registers.d, self),
            Target::E => f(self.registers.e, self),
            Target::F => f(self.registers.f, self),
            Target::H => f(self.registers.h, self),
            Target::L => f(self.registers.l, self),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SRA(reg)));
            }
        }
    }

    fn sub(&mut self, src: Target) {
        let v = match src {
            Target::A => self.registers.a,
            Target::B => self.registers.b,
            Target::C => self.registers.c,
            Target::D => self.registers.d,
            Target::E => self.registers.e,
            Target::F => self.registers.f,
            Target::L => self.registers.l,
            Target::H => self.registers.h,
            Target::HL => self
                .bus
                .read(self.registers.combined_register(src))
                .unwrap(),
            Target::D8 => self.bus.read(self.pc + 1).unwrap(),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SUB(src)));
            }
        };

        // self.registers.set_flag(Flag::Zero, v >= self.registers.a);
        let old = self.registers.a;
        self.registers.a = self.registers.a.wrapping_sub(v);
        // self.registers.set_flag(Flag::Sub, true);
        // self.registers.set_flag(Flag::HalfCarry, false);
        // self.registers.set_flag(Flag::Carry, false);
        self.set_flags(
            Instruction::from_opcode(OpCode::SUB(src)).unwrap(),
            old,
            self.registers.a,
        );
    }

    fn swap(&mut self, reg: Target) {
        let f = |v: u8, cpu: &mut Cpu| {
            let upper = v & (0b1111 << 4);
            let lower = v & 0b1111;
            let old = v;
            let v = (upper >> 4) | (lower << 4);
            cpu.set_flags(Instruction::from_opcode(OpCode::SWAP(reg)).unwrap(), old, v);
            v
        };
        match reg {
            Target::A => self.registers.a = f(self.registers.a, self),
            Target::B => self.registers.b = f(self.registers.b, self),
            Target::C => self.registers.c = f(self.registers.c, self),
            Target::D => self.registers.d = f(self.registers.d, self),
            Target::E => self.registers.e = f(self.registers.e, self),
            Target::F => self.registers.f = f(self.registers.f, self),
            Target::L => self.registers.l = f(self.registers.l, self),
            Target::H => self.registers.h = f(self.registers.h, self),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SWAP(reg)));
            }
        }
    }

    fn xor(&mut self, src: Target) {
        let old = self.registers.a;
        match src {
            Target::A => self.registers.a = 0,
            Target::B => self.registers.a ^= self.registers.b,
            Target::C => self.registers.a ^= self.registers.c,
            Target::D => self.registers.a ^= self.registers.d,
            Target::E => self.registers.a ^= self.registers.e,
            Target::F => self.registers.a ^= self.registers.f,
            Target::L => self.registers.a ^= self.registers.l,
            Target::H => self.registers.a ^= self.registers.h,
            Target::HL => {
                self.registers.a ^= self
                    .bus
                    .read(self.registers.combined_register(src))
                    .unwrap()
            }
            Target::D8 => self.registers.a ^= self.bus.read(self.pc + 1).unwrap(),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::XOR(src)));
            }
        }

        self.set_flags(
            Instruction::from_opcode(OpCode::XOR(src)).unwrap(),
            old,
            self.registers.a,
        );
    }

    #[allow(unused_variables)]
    fn store(&mut self, dst: Target, src: Target) {
        if dst.is_16bit() {
            self.store_16(dst, src);
            return;
        }
    }

    #[allow(unused_variables)]
    fn store_16(&mut self, dst: Target, src: Target) {}

    pub fn set_a(&mut self, value: u8) {
        self.registers.a = value;
    }

    pub fn set_b(&mut self, value: u8) {
        self.registers.b = value;
    }

    #[allow(dead_code)]
    pub fn print_registers(&self) {
        println!("{:#?}", self.registers);
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
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

    cpu.bus.write(cpu.pc + 1, (address1 & 0b11111111) as u8);
    cpu.bus
        .write(cpu.pc + 2, ((address1 & 0b1111111100000000) >> 8) as u8);
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
    cpu.bus.write(cpu.pc + 1, initial + 255);
    cpu.bus.write(cpu.pc + 2, initial + 0);
    cpu.sub(Target::B);
    cpu.jump_by_flag(Flag::Zero);

    assert!(cpu.pc as u8 == initial + 255);
    cpu.reset_registers();

    cpu.registers.b = 5;
    cpu.bus.write(cpu.pc + 1, 0);
    cpu.bus.write(cpu.pc + 2, 0);
    cpu.add(Target::B);
    cpu.jump_by_flag(Flag::NotZero);

    assert!(cpu.pc == 0);
    cpu.reset_registers();

    cpu.registers.a = 200;
    cpu.registers.b = 100;
    cpu.bus.write(cpu.pc + 1, initial + 255);
    cpu.bus.write(cpu.pc + 2, initial + 0);
    cpu.add(Target::B);

    cpu.jump_by_flag(Flag::Carry);

    assert!(cpu.pc as u8 == initial + 255);
    cpu.reset_registers();

    cpu.registers.b = 100;
    cpu.bus.write(cpu.pc + 1, 0);
    cpu.bus.write(cpu.pc + 2, 0);
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

    cpu.bus.write(cpu.pc + 1, initial + 100);
    cpu.registers.a = 255;
    cpu.registers.b = 20;
    cpu.add(Target::B);

    cpu.jr(Flag::Carry);

    assert!(cpu.pc as u8 == initial + 100);
    cpu.reset_registers();
    initial = cpu.pc as u8;

    cpu.bus.write(cpu.pc + 1, initial + 105);
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

    assert!(cpu.bus.read(cpu.sp + 1).unwrap() == 0b10001000);
    assert!(cpu.bus.read(cpu.sp + 2).unwrap() == 0b00010001);

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

    assert_eq!(cpu.registers.a, 2);
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
