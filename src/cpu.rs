use crate::instruction::Target;
extern crate libc;

#[allow(unused_imports)]
use crate::registers::{CARRY_BIT_POS, HALF_CARRY_BIT_POS, SUB_BIT_POS, ZERO_BIT_POS};
use std::mem;

use crate::Flag;
use crate::Instruction;
use crate::Memory;
use crate::OpCode;
use crate::Registers;

use crate::CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES;
use crate::RUN_FLAG;

use std::ptr;

macro_rules! panic_or_print {
    ($a: expr) => {
        if RUN_FLAG & CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES != 0 {
            println!($a);
            return;
        } else {
            panic!($a);
        }
    };

    ($a: expr, $b: expr) => {
        if RUN_FLAG & CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES != 0 {
            println!($a, $b);
            return;
        } else {
            panic!($a, $b);
        }
    };

    ($a: expr, $b: expr, $c: expr) => {
        if RUN_FLAG & CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES != 0 {
            println!($a, $b, $c);
            return;
        } else {
            panic!($a, $b, $c);
        }
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

    pub fn run(&mut self) {
        while self.tick() {}
    }

    pub fn get_reg_a(&self) -> u8 {
        self.registers.a
    }

    pub fn write_to_memory(&mut self, address: u16, byte: u8) {
        self.memory.write_byte(address, byte);
    }

    pub fn read_memory(&self, address: u16) -> u8 {
        self.memory.read_byte(address)
    }

    pub fn zero_memory(&mut self) {
        for i in 0..0xFFFF {
            self.memory.write_byte(i, 0);
        }
    }

    pub fn tick(&mut self) -> bool {
        let instruction_byte = self.memory.read_byte(self.pc);

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
            OpCode::XOR(target) => {
                self.xor(target);
            }
            _ => {
                if RUN_FLAG & CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES != 0 {
                    println!(
                        "Unimplemented {:#?} : {:#x}",
                        instruction.opcode,
                        Instruction::byte_from_opcode(instruction.opcode).unwrap()
                    );
                } else {
                    panic!("Unimplemented");
                }
            }
        }

        self.pc.wrapping_add(pc_increment)
    }

    fn adc(&mut self, reg: Target) {
        let v;
        match reg {
            Target::A => v = self.registers.a as u16,
            Target::B => v = self.registers.b as u16,
            Target::C => v = self.registers.c as u16,
            Target::D => v = self.registers.d as u16,
            Target::E => v = self.registers.e as u16,
            Target::H => v = self.registers.h as u16,
            Target::L => v = self.registers.l as u16,
            Target::HL => v = ((self.registers.h as u16) << 8) | self.registers.l as u16,
            Target::D8 => v = self.memory.read_byte(self.pc + 1) as u16,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::ADC(reg)));
            }
        }

        let carry = self.registers.filter_flag(Flag::Carry) as u16;
        self.registers
            .set_flag(Flag::Zero, self.registers.a as u16 + v + carry == 256);
        self.registers
            .set_flag(Flag::Carry, self.registers.a as u16 + v + carry > 255);
        self.registers.set_flag(
            Flag::HalfCarry,
            self.registers.a < 0b10000 && self.registers.a as u16 + v + carry > 0b1111,
        );
        self.registers.set_flag(Flag::Sub, false);

        self.registers.a = (self.registers.a as u16)
            .wrapping_add(v)
            .wrapping_add(carry) as u8;
    }

    fn add(&mut self, src: Target) {
        let v;

        match src {
            Target::A => v = self.registers.a,
            Target::B => v = self.registers.b,
            Target::C => v = self.registers.c,
            Target::D => v = self.registers.d,
            Target::E => v = self.registers.e,
            Target::F => v = self.registers.f,
            Target::L => v = self.registers.l,
            Target::H => v = self.registers.h,
            Target::HL => {
                v = self
                    .memory
                    .read_byte(self.registers.combined_register(Target::HL))
            }
            Target::D8 => {
                v = self.memory.read_byte(self.pc + 1);
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::ADD(src)));
            }
        }

        self.registers
            .set_flag(Flag::Carry, self.registers.a as i32 + v as i32 > 255);

        self.registers.set_flag(
            Flag::HalfCarry,
            self.registers.a < 0b1111 && self.registers.a + v >= 0b1111,
        );

        self.registers.a = self.registers.a.wrapping_add(v);
        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
        self.registers.set_flag(Flag::Sub, false);
    }

    #[allow(unused_assignments)]
    fn add_16(&mut self, dst: Target, src: Target) {
        let mut v: u16 = 0;
        match src {
            Target::HL => v = ((self.registers.h as u16) << 8) + self.registers.h as u16,
            Target::BC => v = ((self.registers.b as u16) << 8) + self.registers.c as u16,
            Target::DE => v = ((self.registers.d as u16) << 8) + self.registers.e as u16,
            Target::SP => v = self.sp,
            Target::R8 => v = self.memory.read_byte(self.pc + 1) as u16,
            _ => {
                panic_or_print!(
                    "Unimplemented {}",
                    format!("{:#?}", OpCode::ADD16(dst, src))
                );
            }
        }

        match dst {
            Target::HL => {
                self.registers.l = (v & 0xFF) as u8;
                self.registers.h = (v >> 8) as u8;
            }
            Target::SP => {
                self.sp = v;
            }
            _ => {
                panic_or_print!(
                    "Unimplemented {}",
                    format!("{:#?}", OpCode::ADD16(dst, src))
                );
            }
        }
    }

    fn and(&mut self, src: Target) {
        match src {
            Target::A => self.registers.a = self.registers.a & self.registers.a,
            Target::B => self.registers.a = self.registers.a & self.registers.b,
            Target::C => self.registers.a = self.registers.a & self.registers.c,
            Target::D => self.registers.a = self.registers.a & self.registers.d,
            Target::E => self.registers.a = self.registers.a & self.registers.e,
            Target::F => self.registers.a = self.registers.a & self.registers.f,
            Target::L => self.registers.a = self.registers.a & self.registers.l,
            Target::H => self.registers.a = self.registers.a & self.registers.h,
            Target::HL => {
                self.registers.a =
                    self.registers.a & self.memory.read_byte(self.registers.combined_register(src))
            }
            Target::D8 => self.registers.a = self.registers.a & self.memory.read_byte(self.pc + 1),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::AND(src)));
            }
        }
    }

    fn bit(&mut self, bit: u8, reg: Target) {
        let mut v = 0;
        match reg {
            Target::A => v = self.registers.a,
            Target::B => v = self.registers.b,
            Target::C => v = self.registers.c,
            Target::D => v = self.registers.d,
            Target::E => v = self.registers.e,
            Target::H => v = self.registers.h,
            Target::L => v = self.registers.l,
            // Target::HL => v = ((self.registers.h as u16) << 8) | self.registers.l as u16,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::BIT(bit, reg)));
            }
        }
        let bit = v & (1 << bit);

        self.registers.set_flag(Flag::Zero, bit == 0);
        self.registers.set_flag(Flag::Carry, false);
        self.registers.set_flag(Flag::HalfCarry, true);
        self.registers.set_flag(Flag::Sub, false);
    }

    fn call(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.memory
                .write_byte(self.sp - 1, ((self.pc & 0b1111111100000000) >> 8) as u8);
            self.memory
                .write_byte(self.sp, (self.pc & 0b11111111) as u8);

            self.sp = self.sp.wrapping_sub(2);

            self.pc = ((self.memory.read_byte(self.pc + 2) as u16) << 8)
                | self.memory.read_byte(self.pc + 1) as u16;

            return true;
        }

        false
    }

    fn cp(&mut self, reg: Target) {
        let v;
        match reg {
            Target::A => v = self.registers.a as u16,
            Target::B => v = self.registers.b as u16,
            Target::C => v = self.registers.c as u16,
            Target::D => v = self.registers.d as u16,
            Target::E => v = self.registers.e as u16,
            Target::H => v = self.registers.h as u16,
            Target::L => v = self.registers.l as u16,
            Target::HL => v = ((self.registers.h as u16) << 8) | self.registers.l as u16,
            Target::D8 => v = self.memory.read_byte(self.pc + 1) as u16,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::CP(reg)));
            }
        }

        let result = self.registers.a as u16 - v;
        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Sub, true);
        self.registers.set_flag(Flag::HalfCarry, reg == Target::R8);
        self.registers.set_flag(
            Flag::Carry,
            reg == Target::R8 || (self.registers.a as u16).lt(&v),
        );
    }

    fn cpl(&mut self) {
        self.registers.a = !self.registers.a;
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
    }

    fn dec(&mut self, target: Target) {
        let v: *mut u8;
        match target {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::L => v = &mut self.registers.l,
            Target::H => v = &mut self.registers.h,
            Target::HL => {
                let mut v = self
                    .memory
                    .read_byte(self.registers.combined_register(target));
                self.registers.set_flag(Flag::HalfCarry, v == 0b10000);
                v = v.wrapping_sub(1);
                self.registers.set_flag(Flag::Zero, v == 0);
                self.registers.set_flag(Flag::Sub, true);
                self.memory
                    .write_byte(self.registers.combined_register(target), v);
                return;
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::DEC(target)));
            }
        }

        unsafe {
            self.registers.set_flag(Flag::HalfCarry, false);
            self.registers.set_flag(Flag::Carry, false);
            self.registers.set_flag(Flag::Sub, true);

            *v = (*v).wrapping_sub(1);
            self.registers.set_flag(Flag::Zero, *v == 0);
        }
    }

    fn dec_16(&mut self, target: Target) {
        if target == Target::SP {
            self.sp = self.sp.wrapping_sub(1);
            return;
        }

        self.registers.set_combined_register(
            target,
            self.registers.combined_register(target).wrapping_sub(1),
        );
    }

    fn inc(&mut self, target: Target) {
        let v: *mut u8;
        match target {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::L => v = &mut self.registers.l,
            Target::H => v = &mut self.registers.h,
            Target::HL => {
                let mut v = self
                    .memory
                    .read_byte(self.registers.combined_register(target));
                self.registers.set_flag(Flag::HalfCarry, v == 0b1111);
                v = v.wrapping_add(1);
                self.registers.set_flag(Flag::Zero, v == 0);
                self.registers.set_flag(Flag::Sub, false);
                self.memory
                    .write_byte(self.registers.combined_register(target), v);
                return;
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::INC(target)));
            }
        }

        unsafe {
            self.registers.set_flag(Flag::HalfCarry, *v == 0b1111);
            self.registers.set_flag(Flag::Carry, *v == 0b11111111);
            self.registers.set_flag(Flag::Sub, false);

            *v = (*v).wrapping_add(1);
            self.registers.set_flag(Flag::Zero, *v == 0);
        }
    }

    fn inc_16(&mut self, target: Target) {
        if target == Target::SP {
            self.sp = self.sp.wrapping_add(1);
            return;
        }

        self.registers.set_combined_register(
            target,
            self.registers.combined_register(target).wrapping_add(1),
        );
    }

    fn jump(&mut self) {
        self.pc = self.registers.combined_register(Target::HL);
    }

    fn jump_by_flag(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = ((self.memory.read_byte(self.pc + 2) as u16) << 8)
                | self.memory.read_byte(self.pc + 1) as u16;
            return true;
        }
        false
    }

    fn jr(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = (self.pc as i16 + (self.memory.read_byte(self.pc + 1) as i8) as i16) as u16;
            return true;
        }

        false
    }

    fn jump_hl(&mut self) {
        self.pc = self.registers.combined_register(Target::HL);
    }

    #[allow(unused_assignments)]
    pub fn load(&mut self, dst: Target, src: Target) {
        let mut d: *mut u8 = ptr::null_mut();
        let mut v: *mut u8 = ptr::null_mut();

        match dst {
            Target::A => d = &mut self.registers.a,
            Target::B => d = &mut self.registers.b,
            Target::C => d = &mut self.registers.c,
            Target::D => d = &mut self.registers.d,
            Target::E => d = &mut self.registers.e,
            Target::F => d = &mut self.registers.f,
            Target::L => d = &mut self.registers.l,
            Target::H => d = &mut self.registers.h,
            Target::HL | Target::BC | Target::DE => {
                d = self
                    .memory
                    .get_pointer(self.registers.combined_register(dst));
            }
            Target::A8 => {
                d = self
                    .memory
                    .get_pointer(0xFF00 + self.memory.read_byte(self.pc + 1) as u16);
            }
            Target::A16 => {
                d = self.memory.get_pointer(
                    (self.memory.read_byte(self.pc + 1) as u16) << 8
                        | self.memory.read_byte(self.pc + 2) as u16,
                );
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        }

        match src {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::L => v = &mut self.registers.l,
            Target::H => v = &mut self.registers.h,
            Target::HL | Target::BC | Target::DE => {
                v = self
                    .memory
                    .get_pointer(self.registers.combined_register(src))
            }
            Target::A8 => {
                v = self
                    .memory
                    .get_pointer(0xFF00 + self.memory.read_byte(self.pc + 1) as u16);
            }
            Target::A16 => {
                v = self.memory.get_pointer(
                    (self.memory.read_byte(self.pc + 1) as u16) << 8
                        | self.memory.read_byte(self.pc + 2) as u16,
                );
            }
            Target::R8 => {
                v = self
                    .memory
                    .get_pointer(self.pc + self.memory.read_byte(self.pc + 1) as u16)
            }
            Target::D8 => v = self.memory.get_pointer(self.pc + 1),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        }

        unsafe {
            *d = *v;
        }
    }

    pub fn load_16(&mut self, dst: Target, src: Target) {
        let mut v: *mut u16 = ptr::null_mut();
        let mut d: *mut u16 = ptr::null_mut();
        match src {
            Target::A => v = &mut (self.registers.a as u16),
            Target::B => v = &mut (self.registers.b as u16),
            Target::C => v = &mut (self.registers.c as u16),
            Target::D => v = &mut (self.registers.d as u16),
            Target::E => v = &mut (self.registers.e as u16),
            Target::H => v = &mut (self.registers.h as u16),
            Target::L => v = &mut (self.registers.l as u16),
            Target::HL | Target::BC | Target::DE => {
                v = &mut (self.memory.read_byte(self.registers.combined_register(src)) as u16)
            }
            Target::A8 => {
                v = &mut (self
                    .memory
                    .read_byte(0xFF00 + self.memory.read_byte(self.pc + 1) as u16)
                    as u16)
            }
            Target::A16 => {
                v = &mut (self.memory.read_byte(
                    (self.memory.read_byte(self.pc + 1) as u16)
                        << 8 + self.memory.read_byte(self.pc + 2) as u16,
                ) as u16)
            }
            Target::D8 => v = &mut (self.memory.read_byte(self.pc + 1) as u16),
            Target::D16 => unsafe {
                v = libc::malloc(mem::size_of::<u16>()) as *mut u16;

                *v = (((self.memory.read_byte(self.pc + 1) as u16) << 8)
                    + self.memory.read_byte(self.pc + 2) as u16) as u16;
            },
            Target::SP => v = &mut self.sp,
            Target::SP_R8 => {
                v = &mut (self
                    .memory
                    .read_byte(self.sp + self.memory.read_byte(self.pc + 1) as u16)
                    as u16)
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        }

        match dst {
            Target::A => d = &mut (self.registers.a as u16),
            Target::B => d = &mut (self.registers.b as u16),
            Target::C => d = &mut (self.registers.c as u16),
            Target::D => d = &mut (self.registers.d as u16),
            Target::E => d = &mut (self.registers.e as u16),
            Target::H => d = &mut (self.registers.h as u16),
            Target::L => d = &mut (self.registers.l as u16),
            Target::HL | Target::BC | Target::DE => {
                d =
                    self.memory.get_pointer(
                        self.memory.read_byte(self.registers.combined_register(dst)) as u16,
                    ) as *mut u16
            }
            Target::A8 => {
                d = &mut (self
                    .memory
                    .read_byte(0xFF00 + self.memory.read_byte(self.pc + 1) as u16)
                    as u16)
            }
            Target::A16 => {
                d = &mut (self.memory.read_byte(
                    ((self.memory.read_byte(self.pc + 1) as u16) << 8)
                        + self.memory.read_byte(self.pc + 2) as u16,
                ) as u16)
            }
            Target::D8 => v = &mut (self.memory.read_byte(self.pc + 1) as u16),
            Target::D16 => unsafe {
                d = libc::malloc(mem::size_of::<u16>()) as *mut u16;

                *d = (((self.memory.read_byte(self.pc + 1) as u16) << 8)
                    + self.memory.read_byte(self.pc + 2) as u16) as u16;
            },
            Target::SP => d = &mut self.sp,
            Target::SP_R8 => {
                d = &mut (self
                    .memory
                    .read_byte(self.sp + self.memory.read_byte(self.pc + 1) as u16)
                    as u16)
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        }

        unsafe {
            // println!("load_16({:#?}, {:#?})", dst, src);
            // println!("dst = {:p}\tsrc = {:p}", d, v);
            *d = *v;
        }
    }

    fn or(&mut self, src: Target) {
        match src {
            Target::A => self.registers.a = self.registers.a | self.registers.a,
            Target::B => self.registers.a = self.registers.a | self.registers.b,
            Target::C => self.registers.a = self.registers.a | self.registers.c,
            Target::D => self.registers.a = self.registers.a | self.registers.d,
            Target::E => self.registers.a = self.registers.a | self.registers.e,
            Target::F => self.registers.a = self.registers.a | self.registers.f,
            Target::L => self.registers.a = self.registers.a | self.registers.l,
            Target::H => self.registers.a = self.registers.a | self.registers.h,
            Target::HL => {
                self.registers.a =
                    self.registers.a | self.memory.read_byte(self.registers.combined_register(src))
            }
            Target::D8 => self.registers.a = self.registers.a | self.memory.read_byte(self.pc + 1),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::OR(src)));
            }
        }
    }

    fn pop(&mut self, target: Target) {
        let v = ((self.memory.read_byte(self.sp + 1) as u16) << 8)
            | self.memory.read_byte(self.sp + 2) as u16;

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
        let v;
        match target {
            Target::AF | Target::BC | Target::DE | Target::HL => {
                v = self.registers.combined_register(target)
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::PUSH(target)));
            }
        }

        self.memory.write_byte(self.sp, (v & 0b11111111) as u8);
        self.memory
            .write_byte(self.sp - 1, ((v & 0b1111111100000000) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(2);
    }

    fn res(&mut self, bit: u8, reg: Target) {
        let mut v: *mut u8 = ptr::null_mut();
        match reg {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::H => v = &mut self.registers.h,
            Target::L => v = &mut self.registers.l,
            // Target::HL => v = &mut self.registers.a,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RES(bit, reg)));
            }
        }

        let mask = !(1 << bit);
        unsafe {
            *v = *v & mask;
        }
    }

    fn ret(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = ((self.memory.read_byte(self.sp + 1) as u16) << 8)
                | (self.memory.read_byte(self.sp + 2) as u16);
            self.sp = self.sp.wrapping_add(2);

            return true;
        }
        false
    }

    fn rla(&mut self) {
        let msb = (self.registers.a & (1 << ZERO_BIT_POS)) >> ZERO_BIT_POS;
        let carry = self.registers.filter_flag(Flag::Carry);
        self.registers.a = self.registers.a << 1 | carry;
        self.registers.set_flag(Flag::Carry, msb != 0);
        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
    }

    fn rlca(&mut self) {
        self.registers
            .set_flag(Flag::Carry, (self.registers.a & (1 << ZERO_BIT_POS)) != 0);
        self.registers.a = self.registers.a.rotate_left(1);

        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
    }

    fn rra(&mut self) {
        let lsb = self.registers.a & 1;
        let carry = self.registers.filter_flag(Flag::Carry) << ZERO_BIT_POS;
        self.registers.a = self.registers.a >> 1 | carry;
        self.registers.set_flag(Flag::Carry, lsb != 0);
        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
    }

    fn rrca(&mut self) {
        self.registers
            .set_flag(Flag::Carry, (self.registers.a & 1) != 0);
        self.registers.a = self.registers.a.rotate_right(1);

        self.registers.set_flag(Flag::Zero, self.registers.a == 0);
    }

    fn rl(&mut self, reg: Target) {
        match reg {
            Target::A => self.registers.a = self.registers.a.rotate_left(1),
            Target::B => self.registers.b = self.registers.b.rotate_left(1),
            Target::C => self.registers.c = self.registers.c.rotate_left(1),
            Target::D => self.registers.d = self.registers.d.rotate_left(1),
            Target::E => self.registers.e = self.registers.e.rotate_left(1),
            Target::F => self.registers.f = self.registers.f.rotate_left(1),
            Target::H => self.registers.h = self.registers.h.rotate_left(1),
            Target::L => self.registers.l = self.registers.l.rotate_left(1),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RL(reg)));
            }
        }
    }

    fn rlc(&mut self, reg: Target) {
        let mut v: *mut u8 = ptr::null_mut();
        match reg {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::H => v = &mut self.registers.h,
            Target::L => v = &mut self.registers.l,
            Target::HL => {
                let v = self.registers.combined_register(reg);
                self.registers.set_flag(Flag::Carry, (v & (1 << 15)) != 0);
                self.registers.set_combined_register(reg, v.rotate_left(1));
                return;
            }
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RLC(reg)));
            }
        }

        unsafe {
            self.registers
                .set_flag(Flag::Carry, (*v & (1 << ZERO_BIT_POS)) != 0);
            *v = (*v).rotate_left(1);
            self.registers.set_flag(Flag::Zero, *v == 0);
        }
    }

    fn rr(&mut self, reg: Target) {
        match reg {
            Target::A => self.registers.a = self.registers.a.rotate_right(1),
            Target::B => self.registers.b = self.registers.b.rotate_right(1),
            Target::C => self.registers.c = self.registers.c.rotate_right(1),
            Target::D => self.registers.d = self.registers.d.rotate_right(1),
            Target::E => self.registers.e = self.registers.e.rotate_right(1),
            Target::F => self.registers.f = self.registers.f.rotate_right(1),
            Target::H => self.registers.h = self.registers.h.rotate_right(1),
            Target::L => self.registers.l = self.registers.l.rotate_right(1),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RR(reg)));
            }
        }
    }

    #[allow(unused_assignments)]
    fn rrc(&mut self, reg: Target) {
        let mut v: *mut u8 = ptr::null_mut();
        match reg {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::H => v = &mut self.registers.h,
            Target::L => v = &mut self.registers.l,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::RRC(reg)));
            }
        }

        unsafe {
            self.registers.set_flag(Flag::Carry, (*v & 1) != 0);
            *v = (*v).rotate_right(1);
            self.registers.set_flag(Flag::Zero, *v == 0);
        }
    }

    fn rst(&mut self, address: u16) {
        self.memory
            .write_byte(self.sp - 1, ((self.pc & 0b1111111100000000) >> 8) as u8);
        self.memory
            .write_byte(self.sp, (self.pc & 0b11111111) as u8);

        self.sp = self.sp.wrapping_sub(2);

        self.pc = address;
    }

    fn sbc(&mut self, reg: Target) {
        let v;
        match reg {
            Target::A => v = self.registers.a as u16,
            Target::B => v = self.registers.b as u16,
            Target::C => v = self.registers.c as u16,
            Target::D => v = self.registers.d as u16,
            Target::E => v = self.registers.e as u16,
            Target::H => v = self.registers.h as u16,
            Target::L => v = self.registers.l as u16,
            Target::HL => v = ((self.registers.h as u16) << 8) | self.registers.l as u16,
            Target::D8 => v = self.memory.read_byte(self.pc + 1) as u16,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SBC(reg)));
            }
        }

        let carry = self.registers.filter_flag(Flag::Carry) as u16;
        self.registers
            .set_flag(Flag::Zero, self.registers.a as u16 == v);
        self.registers.set_flag(Flag::Sub, true);
        self.registers.set_flag(
            Flag::Carry,
            self.registers.a > 0b1111 && self.registers.a as u16 - v - carry < 0b10000,
        );
        self.registers
            .set_flag(Flag::Zero, (self.registers.a as u16).lt(&(v + carry)));

        self.registers.a = (self.registers.a as u16)
            .wrapping_sub(v)
            .wrapping_sub(carry) as u8;
    }

    fn set(&mut self, bit: u8, reg: Target) {
        let mut v: *mut u8 = ptr::null_mut();
        match reg {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::H => v = &mut self.registers.h,
            Target::L => v = &mut self.registers.l,
            // Target::HL => v = &mut self.registers.h,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SET(bit, reg)));
            }
        }

        unsafe {
            *v = *v | (1 << bit);
        }
    }

    #[allow(dead_code)]
    fn sl(&mut self, reg: Target) {
        match reg {
            Target::A => self.registers.a = self.registers.a << 1,
            Target::B => self.registers.b = self.registers.b << 1,
            Target::C => self.registers.c = self.registers.c << 1,
            Target::D => self.registers.d = self.registers.d << 1,
            Target::E => self.registers.e = self.registers.e << 1,
            Target::F => self.registers.f = self.registers.f << 1,
            Target::H => self.registers.h = self.registers.h << 1,
            Target::L => self.registers.l = self.registers.l << 1,
            _ => {
                panic_or_print!("Unimplemented SL, NOT SURE IF THIS INSTRUCTION EXISTS");
            }
        }
    }

    #[allow(unused_assignments)]
    fn sla(&mut self, reg: Target) {
        let mut v: *mut u8 = ptr::null_mut();
        match reg {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::H => v = &mut self.registers.h,
            Target::L => v = &mut self.registers.l,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SLA(reg)));
            }
        }

        unsafe {
            let bit = (*v) & (1 << ZERO_BIT_POS);
            self.registers.set_flag(Flag::Carry, bit == 0);
            *v = (*v) << 1;
            self.registers.set_flag(Flag::Zero, *v == 0);
            self.registers.set_flag(Flag::HalfCarry, false);
            self.registers.set_flag(Flag::Sub, false);
        }
    }

    fn srl(&mut self, reg: Target) {
        let mut v: *mut u8 = ptr::null_mut();
        match reg {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::H => v = &mut self.registers.h,
            Target::L => v = &mut self.registers.l,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SRL(reg)));
            }
        }

        unsafe {
            let lsb = (*v) & 1;
            *v = *v >> 1;
            self.registers.set_flag(Flag::Zero, *v == 0);
            self.registers.set_flag(Flag::Sub, false);
            self.registers.set_flag(Flag::Carry, lsb != 0);
            self.registers.set_flag(Flag::HalfCarry, false);
        }
    }

    #[allow(unused_assignments)]
    fn sra(&mut self, reg: Target) {
        let mut v: *mut u8 = ptr::null_mut();
        match reg {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::H => v = &mut self.registers.h,
            Target::L => v = &mut self.registers.l,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SRA(reg)));
            }
        }

        unsafe {
            let lsb = (*v) & 1;
            let msb = (*v) & (1 << ZERO_BIT_POS);
            *v = ((*v) >> 1) | msb;
            self.registers.set_flag(Flag::Zero, *v == 0);
            self.registers.set_flag(Flag::Sub, false);
            self.registers.set_flag(Flag::Carry, lsb != 0);
            self.registers.set_flag(Flag::HalfCarry, false);
        }
    }

    fn sub(&mut self, src: Target) {
        let v;
        match src {
            Target::A => v = self.registers.a,
            Target::B => v = self.registers.b,
            Target::C => v = self.registers.c,
            Target::D => v = self.registers.d,
            Target::E => v = self.registers.e,
            Target::F => v = self.registers.f,
            Target::L => v = self.registers.l,
            Target::H => v = self.registers.h,
            Target::HL => v = self.memory.read_byte(self.registers.combined_register(src)),
            Target::D8 => v = self.memory.read_byte(self.pc + 1),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SUB(src)));
            }
        }

        self.registers.set_flag(Flag::Zero, v >= self.registers.a);

        self.registers.a = self.registers.a.wrapping_sub(v);
        self.registers.set_flag(Flag::Sub, true);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, false);
    }

    fn swap(&mut self, reg: Target) {
        let v: *mut u8;
        match reg {
            Target::A => v = &mut self.registers.a,
            Target::B => v = &mut self.registers.b,
            Target::C => v = &mut self.registers.c,
            Target::D => v = &mut self.registers.d,
            Target::E => v = &mut self.registers.e,
            Target::F => v = &mut self.registers.f,
            Target::L => v = &mut self.registers.l,
            Target::H => v = &mut self.registers.h,
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::SWAP(reg)));
            }
        }

        unsafe {
            let upper = *v & (0b1111 << 4);
            let lower = *v & 0b1111;
            *v = (upper >> 4) | (lower << 4);
        }
    }

    // fn add_hl(&mut self, src: Target) {
    //     match src {
    //         Target::A => self.registers.hl = self.registers.hl.wrapping_add(self.registers.a),
    //         Target::B => self.registers.hl = self.registers.hl.wrapping_add(self.registers.b),
    //         Target::C => self.registers.hl = self.registers.hl.wrapping_add(self.registers.c),
    //         Target::D => self.registers.hl = self.registers.hl.wrapping_add(self.registers.d),
    //         Target::E => self.registers.hl = self.registers.hl.wrapping_add(self.registers.e),
    //         Target::F => self.registers.hl = self.registers.hl.wrapping_add(self.registers.f),
    //         Target::L => self.registers.hl = self.registers.hl.wrapping_add(self.registers.l),
    //         Target::H => self.registers.hl = self.registers.hl.wrapping_add(self.registers.h),
    //         _ => {
    //             panic_or_print!("Unimplemented")
    //         }
    //     }
    // }

    fn xor(&mut self, src: Target) {
        match src {
            Target::A => self.registers.a = self.registers.a ^ self.registers.a,
            Target::B => self.registers.a = self.registers.a ^ self.registers.b,
            Target::C => self.registers.a = self.registers.a ^ self.registers.c,
            Target::D => self.registers.a = self.registers.a ^ self.registers.d,
            Target::E => self.registers.a = self.registers.a ^ self.registers.e,
            Target::F => self.registers.a = self.registers.a ^ self.registers.f,
            Target::L => self.registers.a = self.registers.a ^ self.registers.l,
            Target::H => self.registers.a = self.registers.a ^ self.registers.h,
            Target::HL => {
                self.registers.a =
                    self.registers.a ^ self.memory.read_byte(self.registers.combined_register(src))
            }
            Target::D8 => self.registers.a = self.registers.a ^ self.memory.read_byte(self.pc + 1),
            _ => {
                panic_or_print!("Unimplemented {}", format!("{:#?}", OpCode::XOR(src)));
            }
        }
    }

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

    let mut initial = cpu.pc as u8;

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

    assert!(cpu.memory.read_byte(cpu.sp + 1) == 0b10001000);
    assert!(cpu.memory.read_byte(cpu.sp + 2) == 0b00010001);

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
