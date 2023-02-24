use crate::instruction::Target;

#[allow(unused_imports)]
use crate::registers::{CARRY_BIT_POS, HALF_CARRY_BIT_POS, SUB_BIT_POS, ZERO_BIT_POS};

use crate::Flag;
use crate::Instruction;
use crate::Memory;
use crate::OpCode;
use crate::Registers;

use std::ptr;

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    memory: Memory,
    pc: u16,
    sp: u16,
    addr: u16,
    is_prefixed: bool,
}

#[allow(dead_code, unused_assignments)]
impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            memory: Memory::new(),
            pc: 0,
            sp: 0,
            addr: 0,
            is_prefixed: false,
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

        match instruction.opcode.to_owned() {
            OpCode::BIT(bit, target) => {
                self.bit(bit, target);
            }
            OpCode::SET(bit, target) => {
                self.set(bit, target);
            }
            OpCode::RES(bit, target) => {
                self.res(bit, target);
            }
            OpCode::LD(dst, src) => {
                self.load(dst, src);
                if src == Target::D8 {
                    pc_increment = 2;
                } else {
                    pc_increment = 1;
                }
            }
            OpCode::ADD(target) => {
                self.add(target);
            }
            OpCode::Add16(dst, src) => {
                self.add_16(dst, src);
            }
            OpCode::ADDHL(target) => {
                self.add_hl(target);
            }
            OpCode::ADC(_target, _src) => {
                todo!();
            }
            OpCode::SUB(target) => {
                self.sub(target);
            }
            OpCode::SBC(_target, _src) => {
                todo!();
            }
            OpCode::CP(_target) => {
                todo!();
            }
            OpCode::AND(target) => {
                self.and(target);
            }
            OpCode::OR(target) => {
                self.or(target);
            }
            OpCode::XOR(target) => {
                self.xor(target);
            }
            OpCode::INC(target) => {
                self.inc(target);
            }
            OpCode::DEC(target) => {
                self.dec(target);
            }
            OpCode::CCF => {
                self.registers
                    .set_flag(Flag::Carry, !self.registers.get_flag(Flag::Zero));
            }
            OpCode::SCF => {
                self.registers.set_flag(Flag::Carry, true);
            }
            OpCode::RRA => {
                self.registers.rotate_right(Target::A);
            }
            OpCode::RLA => {
                self.registers.rotate_left(Target::A);
            }
            OpCode::RR(target) => {
                self.rr(target);
            }
            OpCode::RL(target) => {
                self.rl(target);
            }
            OpCode::RRC(target) => {
                self.rrc(target);
            }
            OpCode::RLC(target) => {
                self.rlc(target);
            }
            OpCode::SRA(target) => {
                self.sra(target);
            }
            OpCode::SLA(target) => {
                self.sla(target);
            }
            OpCode::SRL(target) => {
                self.sr(target);
            }
            // OpCode::SLL(target) => { Seems like this instruction does not exist
            //     self.sl(target);
            // }
            OpCode::CPL(target) => {
                self.cpl(target);
            }
            OpCode::SWAP(target) => {
                self.swap(target);
            }
            OpCode::JUMP(flag) => {
                let value = self.memory.read_byte(self.pc + 1);
                if self.jump_by_flag(flag, value as u16) {
                    pc_increment = 0;
                }
            }
            OpCode::JumpUnconditional => {
                self.jump(self.addr);
            }
            OpCode::CALL(flag) => match flag {
                Flag::Zero => if self.registers.get_flag(flag) {},
                _ => {}
            },
            OpCode::PREFIX => {
                self.is_prefixed = true;
            }
            _ => {
                panic!("Unimplemented");
            }
        }

        self.pc.wrapping_add(pc_increment)
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
                panic!("Unimplemented");
            }
        }
        let bit = v & (1 << bit);

        self.registers.set_flag(Flag::Zero, bit != 0);
        self.registers.set_flag(Flag::Carry, false);
        self.registers.set_flag(Flag::HalfCarry, true);
        self.registers.set_flag(Flag::Sub, false);
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
                panic!("Unimplemented");
            }
        }

        unsafe {
            *v = *v | (1 << bit);
        }
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
                panic!("Unimplemented");
            }
        }

        let mask = !(1 << bit);
        unsafe {
            *v = *v & mask;
        }
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
            Target::A8 => {
                self.load_16(dst, src);
                return;
            }
            Target::A16 => {
                self.load_16(dst, src);
                return;
            }
            _ => {
                panic!("Unimplemented");
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
            Target::D8 => v = &mut self.memory.read_byte(self.pc + 1),
            Target::A8 => {
                self.load_16(dst, src);
                return;
            }
            Target::A16 => {
                self.load_16(dst, src);
                return;
            }
            _ => {
                panic!("Unimplemented");
            }
        }

        unsafe {
            *d = *v;
        }
    }

    pub fn load_16(&mut self, dst: Target, src: Target) {
        match dst {
            Target::A8 => match src {
                Target::A => self.addr = self.registers.a as u16,
                _ => {
                    panic!("Unimplemented")
                }
            },
            Target::A => match src {
                Target::A8 => self.registers.a = (self.addr & 0b1111) as u8,
                Target::A16 => self.registers.a = ((self.addr & 0b11110000) >> 4) as u8,
                _ => {
                    panic!("Unimplemented");
                }
            },
            _ => {
                panic!("Unimplemented");
            }
        }
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
            Target::D8 => {
                v = self.memory.read_byte(self.pc + 1);
            }
            _ => {
                panic!("Invalid Target");
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
                panic!("Unimplemented")
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
                panic!("Unimplemented");
            }
        }
    }

    fn add_hl(&mut self, src: Target) {
        match src {
            Target::A => self.registers.hl = self.registers.hl.wrapping_add(self.registers.a),
            Target::B => self.registers.hl = self.registers.hl.wrapping_add(self.registers.b),
            Target::C => self.registers.hl = self.registers.hl.wrapping_add(self.registers.c),
            Target::D => self.registers.hl = self.registers.hl.wrapping_add(self.registers.d),
            Target::E => self.registers.hl = self.registers.hl.wrapping_add(self.registers.e),
            Target::F => self.registers.hl = self.registers.hl.wrapping_add(self.registers.f),
            Target::L => self.registers.hl = self.registers.hl.wrapping_add(self.registers.l),
            Target::H => self.registers.hl = self.registers.hl.wrapping_add(self.registers.h),
            _ => {
                panic!("Unimplemented")
            }
        }
    }

    fn cpl(&mut self, reg: Target) {
        match reg {
            Target::A => self.registers.a = !self.registers.a,
            Target::B => self.registers.b = !self.registers.b,
            Target::C => self.registers.c = !self.registers.c,
            Target::D => self.registers.d = !self.registers.d,
            Target::E => self.registers.e = !self.registers.e,
            Target::F => self.registers.f = !self.registers.f,
            Target::L => self.registers.l = !self.registers.l,
            Target::H => self.registers.h = !self.registers.h,
            _ => {
                panic!("Unimplemented");
            }
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
            _ => {
                panic!("Invalid Target");
            }
        }

        self.registers.set_flag(Flag::Zero, v >= self.registers.a);

        self.registers.a = self.registers.a.wrapping_sub(v);
        self.registers.set_flag(Flag::Sub, true);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, false);
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
            Target::HL => todo!(),
            Target::HLP => todo!(),
            Target::HLM => todo!(),
            Target::BC => {
                todo!();
            }
            Target::DE => {
                todo!();
            }
            _ => {
                panic!("Unimplemented")
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
            Target::HL => todo!(),
            Target::HLP => todo!(),
            Target::HLM => todo!(),
            Target::BC => {
                todo!();
            }
            Target::DE => {
                todo!();
            }
            _ => {
                panic!("Unimplemented")
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
            _ => {
                panic!("Unimplemented");
            }
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
            _ => {
                panic!("Unimplemented");
            }
        }
    }

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
            _ => {
                panic!("Unimplemented");
            }
        }
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
                panic!("Unimplemented")
            }
        }

        unsafe {
            let upper = *v & (0b1111 << 4);
            let lower = *v & 0b1111;
            *v = (upper >> 4) | (lower << 4);
        }
    }

    fn jump_by_flag(&mut self, flag: Flag, address: u16) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = address;
            return true;
        }
        false
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
                panic!("Unimplemented");
            }
        }
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
                panic!("Unimplemented");
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
                panic!("Unimplemented");
            }
        }

        unsafe {
            let bit: u8 = (*v) & 1;
            *v = (*v) >> 1;
            let flag_bit = self.registers.filter_flag(Flag::Carry);
            *v = (*v) | (flag_bit << ZERO_BIT_POS);
            self.registers.set_flag_from_u8(Flag::Carry, bit);
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
            _ => {
                panic!("Unimplemented");
            }
        }

        unsafe {
            let bit: u8 = (*v) & 0b10000000;
            *v = (*v) << 1;
            let flag_bit = self.registers.filter_flag(Flag::Carry);
            *v = (*v) | flag_bit;
            self.registers.set_flag_from_u8(Flag::Carry, bit);
        }
    }

    fn sr(&mut self, reg: Target) {
        match reg {
            Target::A => self.registers.a = self.registers.a >> 1,
            Target::B => self.registers.b = self.registers.b >> 1,
            Target::C => self.registers.c = self.registers.c >> 1,
            Target::D => self.registers.d = self.registers.d >> 1,
            Target::E => self.registers.e = self.registers.e >> 1,
            Target::F => self.registers.f = self.registers.f >> 1,
            Target::H => self.registers.h = self.registers.h >> 1,
            Target::L => self.registers.l = self.registers.l >> 1,
            _ => {
                panic!("Unimplemented");
            }
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
                panic!("Unimplemented");
            }
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
                panic!("Unimplemented");
            }
        }

        unsafe {
            let bit = (*v) & (1 << ZERO_BIT_POS);
            *v = ((*v) >> 1) | bit;
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
                panic!("Unimplemented");
            }
        }

        unsafe {
            let bit = (*v) & 1;
            *v = ((*v) << 1) | bit;
        }
    }

    fn jump(&mut self, address: u16) {
        self.pc = address;
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
fn test_increment() {
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
fn test_decrement() {
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
fn test_and() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 3;
    cpu.registers.b = 5;
    cpu.and(Target::B);

    assert!(cpu.registers.a == 1);
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
fn test_xor() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 3;
    cpu.registers.b = 5;
    cpu.xor(Target::B);

    assert!(cpu.registers.a == 6);
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
fn test_swap() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 128 + 4;
    cpu.swap(Target::A);
    assert!(72 == cpu.registers.a);
}

#[test]
fn test_jump() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 5;
    cpu.registers.b = 5;
    cpu.sub(Target::B);
    cpu.jump_by_flag(Flag::Zero, 255);

    assert!(cpu.pc == 255);
}

#[test]
fn test_load() {
    let mut cpu = Cpu::new();
    cpu.write_to_memory(
        0,
        Instruction::byte_from_opcode(OpCode::LD(Target::A, Target::D8)).unwrap(),
    );
    cpu.write_to_memory(1, 100);
    cpu.tick();
    cpu.tick();
    assert!(cpu.registers.a == 100);
}

#[test]
fn test_cpl() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.cpl(Target::A);
    assert!(cpu.registers.a == 0b11111110);
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
fn test_rrc() {
    let mut cpu = Cpu::new();

    cpu.registers.set_flag(Flag::Carry, true);
    cpu.registers.a = 0;
    cpu.rrc(Target::A);

    assert!(cpu.registers.a == 0b10000000);
    assert!(!cpu.registers.get_flag(Flag::Carry));

    cpu.registers.a = 1;
    cpu.rrc(Target::A);

    assert!(cpu.registers.a == 0);
    assert!(cpu.registers.get_flag(Flag::Carry));
}

#[test]
fn test_rlc() {
    let mut cpu = Cpu::new();

    cpu.registers.set_flag(Flag::Carry, true);
    cpu.registers.a = 0;
    cpu.rlc(Target::A);

    assert!(cpu.registers.a == 1);
    assert!(!cpu.registers.get_flag(Flag::Carry));

    cpu.registers.a = 0b10000000;
    cpu.rlc(Target::A);

    assert!(cpu.registers.a == 0);
    assert!(cpu.registers.get_flag(Flag::Carry));
}

#[test]
fn test_sr() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 2;
    cpu.sr(Target::A);

    assert!(cpu.registers.a == 1);

    cpu.sr(Target::A);

    assert!(cpu.registers.a == 0);
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
fn test_sra() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 0b10000000;
    cpu.sra(Target::A);

    assert!(cpu.registers.a == 0b11000000);
}

#[test]
fn test_sla() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.sla(Target::A);

    assert!(cpu.registers.a == 3);
}

#[test]
fn test_bit() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 0b10000000;
    cpu.bit(7, Target::A);

    assert!(cpu.registers.get_flag(Flag::Zero));
    assert!(!cpu.registers.get_flag(Flag::Sub));
    assert!(!cpu.registers.get_flag(Flag::Carry));
    assert!(cpu.registers.get_flag(Flag::HalfCarry));
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
fn test_res() {
    let mut cpu = Cpu::new();

    cpu.registers.a = 1;
    cpu.res(0, Target::A);

    assert!(cpu.registers.a == 0);
}
