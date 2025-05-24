use crate::consoles::bus::Bus;
use crate::consoles::readable::Readable;
use crate::consoles::writeable::Writeable;
#[allow(unused_imports)]
use crate::utils::conversion::u16_to_u8;

use super::game_boy::GbBus;
use super::target::Target;
extern crate libc;

#[allow(unused_imports)]
use super::registers::{CARRY_BIT_POS, HALF_CARRY_BIT_POS, SUB_BIT_POS, ZERO_BIT_POS};
use std::cell::RefCell;
use std::rc::Rc;

use super::instruction::{FlagAction, Instruction};
use super::opcode::OpCode;
use super::registers::Flag;
use super::registers::Registers;

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    bus: Rc<RefCell<GbBus>>,
    pc: u16,
    sp: u16,
    addr: u16,
    is_prefixed: bool,
    interrupts_enabled: bool,
    is_stopped: bool,
}

#[allow(dead_code, unused_assignments)]
impl Cpu {
    pub fn new(bus: Rc<RefCell<Bus<u16, u8, u16>>>) -> Cpu {
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
            let _ = self.bus.borrow_mut().write(i as u16, program[i]);
        }
    }

    fn set_flags(&mut self, instruction: &Instruction, old_value: u8, new_value: u8) {
        match instruction.flag_affection.zero_flag {
            FlagAction::Reset => self.registers.set_flag(Flag::Zero, false),
            FlagAction::Set => self.registers.set_flag(Flag::Zero, true),
            FlagAction::Affected => self.registers.set_flag(Flag::Zero, new_value == 0),
            _ => {}
        }

        match instruction.flag_affection.sub_flag {
            FlagAction::Reset => self.registers.set_flag(Flag::Sub, false),
            FlagAction::Set => self.registers.set_flag(Flag::Sub, true),
            FlagAction::Affected => {
                // Check if this arm ever gets executed
                unimplemented!()
            }
            _ => {}
        }

        match instruction.flag_affection.half_carry_flag {
            FlagAction::Reset => self.registers.set_flag(Flag::HalfCarry, false),
            FlagAction::Set => self.registers.set_flag(Flag::HalfCarry, true),
            FlagAction::Affected => {
                match instruction.flag_affection.sub_flag {
                    FlagAction::Affected => self
                        .registers
                        .set_flag(Flag::HalfCarry, old_value > 0b1111 && new_value < 0b10000),
                    FlagAction::Set => self
                        .registers
                        .set_flag(Flag::HalfCarry, old_value > 0b1111 && new_value < 0b10000),
                    FlagAction::Reset => self
                        .registers
                        .set_flag(Flag::HalfCarry, old_value <= 0b1111 && new_value > 0b1111),
                    _ => {}
                }
                // if instruction.[SUB_FLAG] == AFFECTED || instruction.flags[SUB_FLAG] == SET {
                //     self.registers
                //         .set_flag(Flag::HalfCarry, old_value > 0b1111 && new_value < 0b10000);
                // } else if instruction.flags[SUB_FLAG] == RESET {
                //     self.registers
                //         .set_flag(Flag::HalfCarry, old_value < 0b10000 && new_value > 0b1111);
                // }
            }
            _ => {}
        }

        match instruction.flag_affection.carry_flag {
            FlagAction::Reset => self.registers.set_flag(Flag::Carry, false),
            FlagAction::Set => self.registers.set_flag(Flag::Carry, true),
            FlagAction::Affected => {
                match instruction.flag_affection.sub_flag {
                    FlagAction::Affected => {
                        self.registers.set_flag(Flag::Carry, old_value < new_value)
                    }
                    FlagAction::Set => self.registers.set_flag(Flag::Carry, old_value < new_value),
                    FlagAction::Reset => {
                        self.registers.set_flag(Flag::Carry, old_value > new_value);
                        println!("old = {old_value} new = {new_value}");
                    }
                    _ => {}
                }
                // if let FlagAction::Affected = instruction.flag_affection.sub_flag {
                //     self.registers .set_flag(Flag::Carry, old_value > 0b0 && old_value < new_value);

                // } else if let FlagAction::Set = instruction.flag_affection.sub_flag {
                //     self.registers
                //         .set_flag(Flag::Carry, old_value > 0b0 && old_value < new_value);
                // } else if let FlagAction::Reset =  instruction.flag_affection.sub_flag {
                //     self.registers
                //         .set_flag(Flag::Carry, old_value < 0b11111111 && old_value > new_value);
                // }
            }
            _ => {}
        }
    }

    fn set_flags_16(&mut self, instruction: &Instruction, old_value: u16, new_value: u16) {
        match instruction.flag_affection.zero_flag {
            FlagAction::Reset => self.registers.set_flag(Flag::Zero, false),
            FlagAction::Set => self.registers.set_flag(Flag::Zero, true),
            FlagAction::Affected => self.registers.set_flag(Flag::Zero, new_value == 0),
            _ => {}
        }

        match instruction.flag_affection.sub_flag {
            FlagAction::Reset => self.registers.set_flag(Flag::Sub, false),
            FlagAction::Set => self.registers.set_flag(Flag::Sub, true),
            FlagAction::Affected => {
                // Check if this arm ever gets executed
            }
            _ => {}
        }

        match instruction.flag_affection.half_carry_flag {
            FlagAction::Reset => self.registers.set_flag(Flag::HalfCarry, false),
            FlagAction::Set => self.registers.set_flag(Flag::HalfCarry, true),
            FlagAction::Affected => {
                match instruction.flag_affection.sub_flag {
                    FlagAction::Affected => self
                        .registers
                        .set_flag(Flag::HalfCarry, old_value > 0b1111 && new_value <= 0b1111),
                    FlagAction::Set => self
                        .registers
                        .set_flag(Flag::HalfCarry, old_value > 0b1111 && new_value <= 0b1111),
                    FlagAction::Reset => self.registers.set_flag(Flag::HalfCarry, false),
                    FlagAction::NotAffected => self
                        .registers
                        .set_flag(Flag::HalfCarry, old_value > 0b1111 && new_value <= 0b1111),
                }
                // if instruction.flags[SUB_FLAG] == AFFECTED || instruction.flags[SUB_FLAG] == SET {
                //     self.registers.set_flag(
                //         Flag::HalfCarry,
                //         old_value > 0b11111111 && new_value < 0b100000000,
                //     );
                // } else if instruction.flags[SUB_FLAG] == RESET {
                //     self.registers.set_flag(
                //         Flag::HalfCarry,
                //         old_value < 0b100000000 && new_value > 0b11111111,
                //     );
                // }
            }
            _ => {}
        }

        match instruction.flag_affection.carry_flag {
            FlagAction::Reset => self.registers.set_flag(Flag::Carry, false),
            FlagAction::Set => self.registers.set_flag(Flag::Carry, true),
            FlagAction::Affected => {
                match instruction.flag_affection.sub_flag {
                    FlagAction::Affected => self
                        .registers
                        .set_flag(Flag::Carry, old_value > 0b0 && old_value < new_value),
                    FlagAction::Set => self.registers.set_flag(Flag::Carry, true),
                    FlagAction::Reset => self.registers.set_flag(Flag::Carry, false),
                    _ => {}
                }
                // if instruction.flags[SUB_FLAG] == AFFECTED || instruction.flags[SUB_FLAG] == SET {
                //     self.registers
                //         .set_flag(Flag::Carry, old_value > 0b0 && old_value < new_value);
                // } else if instruction.flags[SUB_FLAG] == RESET {
                //     self.registers.set_flag(
                //         Flag::Carry,
                //         old_value < 0b1111111111111111 && old_value > new_value,
                //     );
                // }
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
        let _ = self.bus.borrow_mut().write(address, byte);
    }

    pub fn read_memory(&self, address: u16) -> u8 {
        self.bus.borrow().read(address).unwrap()
    }

    pub fn zero_memory(&mut self) {
        for i in 0..0xFFFF {
            let _ = self.bus.borrow_mut().write(i, 0);
        }
    }

    pub fn set_memory_to_end_of_program(&mut self) {
        for i in 0..0xFFFF {
            let _ = self.bus.borrow_mut().write(
                i,
                Instruction::byte_from_opcode(OpCode::EndOfProgram).unwrap(),
            );
        }
    }

    pub fn tick(&mut self) -> bool {
        let instruction_byte = self.bus.borrow().read(self.pc).unwrap();

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
                self.jp();
            }
            OpCode::JR(flag) => {
                self.jr(flag);
            }
            OpCode::JP_HL => {
                self.jump_hl();
            }
            OpCode::JRUC => {
                self.jruc();
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
                .borrow()
                .read(self.registers.combined_register(reg))
                .unwrap(),
            Target::D8 => self.bus.borrow().read(self.pc + 1).unwrap(),
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::ADC(reg)));
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
        self.registers.a = self.registers.a.wrapping_add(v + carry);

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
                .borrow()
                .read(self.registers.combined_register(Target::HL))
                .unwrap(),
            Target::D8 => self.bus.borrow().read(self.pc + 1).unwrap(),
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::ADD(src)));
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

    fn add_16(&mut self, dst: Target, src: Target) {
        let v: u16 = match src {
            Target::HL => self.registers.combined_register(src),
            Target::BC => self.registers.combined_register(src),
            Target::DE => self.registers.combined_register(src),
            Target::SP => self.sp,
            Target::R8 => self.bus.borrow().read(self.pc + 1).unwrap() as u16,
            _ => {
                unimplemented!(
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
                unimplemented!(
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
                    .borrow()
                    .read(self.registers.combined_register(src))
                    .unwrap()
            }
            Target::D8 => self.registers.a &= self.bus.borrow().read(self.pc + 1).unwrap(),
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::AND(src)));
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
            Target::HL => self
                .bus
                .borrow()
                .read(self.registers.combined_register(Target::HL))
                .unwrap(),
            _ => {
                unimplemented!(
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
                .borrow_mut()
                .write(self.sp - 1, ((self.pc & 0b1111111100000000) >> 8) as u8);
            let _ = self
                .bus
                .borrow_mut()
                .write(self.sp, (self.pc & 0b11111111) as u8);

            self.sp = self.sp.wrapping_sub(2);

            self.pc = ((self.bus.borrow().read(self.pc + 2).unwrap() as u16) << 8)
                | self.bus.borrow().read(self.pc + 1).unwrap() as u16;

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
            Target::D8 => self.bus.borrow().read(self.pc + 1).unwrap() as u16,
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::CP(reg)));
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
                let mut v = self.registers.combined_register(target);
                // self.registers.set_flag(Flag::HalfCarry, v == 0b10000);
                let old = v;
                v = v.wrapping_sub(1);
                self.registers.set_combined_register(Target::HL, v);
                // self.registers.set_flag(Flag::Zero, v == 0);
                // self.registers.set_flag(Flag::Sub, true);
                self.set_flags_16(
                    Instruction::from_opcode(OpCode::DEC(target)).unwrap(),
                    old,
                    v,
                );
            }
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::DEC(target)));
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
                let mut v = self.registers.combined_register(target);
                let old = v;
                // self.registers.set_flag(Flag::HalfCarry, v == 0b1111);
                v = v.wrapping_add(1);
                self.registers.set_combined_register(Target::HL, v);
                // self.registers.set_flag(Flag::Zero, v == 0);
                // self.registers.set_flag(Flag::Sub, false);
                self.set_flags_16(
                    Instruction::from_opcode(OpCode::INC(target)).unwrap(),
                    old,
                    v,
                );
            }
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::INC(target)));
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

    fn jp(&mut self) {
        let lower_byte = self.bus.borrow().read(self.pc + 1).unwrap() as u16;
        let upper_byte = self.bus.borrow().read(self.pc + 2).unwrap() as u16;
        self.pc = (upper_byte << 8) + lower_byte
    }

    fn jump_by_flag(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = ((self.bus.borrow().read(self.pc + 2).unwrap() as u16) << 8)
                | self.bus.borrow().read(self.pc + 1).unwrap() as u16;
            true
        } else {
            false
        }
    }

    fn jruc(&mut self) {
        self.pc = self.pc + self.bus.borrow().read(self.pc + 1).unwrap() as u16;
    }

    fn jr(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = self.pc + self.bus.borrow().read(self.pc + 1).unwrap() as u16;
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
                .borrow()
                .read(self.registers.combined_register(src))
                .unwrap(),
            Target::A8 => self
                .bus
                .borrow()
                .read(0xFF00 + self.bus.borrow().read(self.pc + 1).unwrap() as u16)
                .unwrap(),
            Target::A16 => self
                .bus
                .borrow()
                .read(
                    (self.bus.borrow().read(self.pc + 1).unwrap() as u16) << 8
                        | self.bus.borrow().read(self.pc + 2).unwrap() as u16,
                )
                .unwrap(),
            Target::R8 => self
                .bus
                .borrow()
                .read(self.pc + self.bus.borrow().read(self.pc + 1).unwrap() as u16)
                .unwrap(),
            Target::D8 => self.bus.borrow().read(self.pc + 1).unwrap(),
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
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
            Target::HL => self.registers.set_combined_register(Target::HL, v as u16),
            Target::BC => self.registers.set_combined_register(Target::BC, v as u16),
            Target::DE => self.registers.set_combined_register(Target::DE, v as u16),
            Target::A8 => {
                let mut bus = self.bus.borrow_mut();
                let addr = 0xFF00 + bus.read(self.pc + 1).unwrap() as u16;
                let _ = bus.write(addr, v);
            }
            Target::A16 => {
                let mut bus = self.bus.borrow_mut();
                let addr = (bus.read(self.pc + 1).unwrap() as u16) << 8
                    | bus.read(self.pc + 2).unwrap() as u16;
                let _ = bus.write(addr, v);
            }
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
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
                .borrow()
                .read(self.registers.combined_register(src))
                .unwrap() as u16,
            Target::A8 => self
                .bus
                .borrow()
                .read(0xFF00 + self.bus.borrow().read(self.pc + 1).unwrap() as u16)
                .unwrap() as u16,
            Target::A16 => self
                .bus
                .borrow()
                .read(
                    ((self.bus.borrow().read(self.pc + 1).unwrap() as u16) << 8)
                        + self.bus.borrow().read(self.pc + 2).unwrap() as u16,
                )
                .unwrap() as u16,
            Target::D8 => self.bus.borrow().read(self.pc + 1).unwrap() as u16,
            Target::D16 => {
                ((self.bus.borrow().read(self.pc + 1).unwrap() as u16) << 8)
                    + (self.bus.borrow().read(self.pc + 2).unwrap() as u16)
            }
            Target::SP => self.sp,
            Target::SP_R8 => self
                .bus
                .borrow()
                .read(self.sp + self.bus.borrow().read(self.pc + 1).unwrap() as u16)
                .unwrap() as u16,
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        };

        match dst {
            Target::HL | Target::BC | Target::DE => {
                let _ = self.bus.borrow_mut().write_16(
                    self.bus
                        .borrow()
                        .read(self.registers.combined_register(dst))
                        .unwrap() as u16,
                    v,
                );
            }
            Target::A8 => {
                let mut bus = self.bus.borrow_mut();
                let addr = 0xFF00 + bus.read(self.pc + 1).unwrap() as u16;
                let _ = bus.write_16(addr, v);
            }
            Target::A16 => {
                let _ = self.bus.borrow_mut().write_16(
                    ((self.bus.borrow().read(self.pc + 1).unwrap() as u16) << 8)
                        + self.bus.borrow().read(self.pc + 2).unwrap() as u16,
                    v,
                );
            }
            Target::SP => self.sp = v,
            Target::SP_R8 => {
                let mut bus = self.bus.borrow_mut();
                let addr = self.sp + bus.read(self.pc + 1).unwrap() as u16;
                let _ = bus.write_16(addr, v);
            }
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::LD(dst, src)));
            }
        }
    }

    fn or(&mut self, src: Target) {
        let old = self.registers.a;
        match src {
            Target::A => self.registers.a |= self.registers.a,
            Target::B => self.registers.a |= self.registers.b,
            Target::C => self.registers.a |= self.registers.c,
            Target::D => self.registers.a |= self.registers.d,
            Target::E => self.registers.a |= self.registers.e,
            Target::F => self.registers.a |= self.registers.f,
            Target::L => self.registers.a |= self.registers.l,
            Target::H => self.registers.a |= self.registers.h,
            Target::HL => {
                self.registers.a =
                    (self.registers.a as u16 | self.registers.combined_register(Target::HL)) as u8
            }
            Target::D8 => self.registers.a |= self.bus.borrow().read(self.pc + 1).unwrap(),
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::OR(src)));
            }
        }

        self.set_flags(
            Instruction::from_opcode(OpCode::OR(src)).unwrap(),
            old,
            self.registers.a,
        );
    }

    fn pop(&mut self, target: Target) {
        let v = ((self.bus.borrow().read(self.sp + 1).unwrap() as u16) << 8)
            | self.bus.borrow().read(self.sp + 2).unwrap() as u16;

        self.sp = self.sp.wrapping_add(2);

        match target {
            Target::AF | Target::BC | Target::DE | Target::HL => {
                self.registers.set_combined_register(target, v)
            }
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::POP(target)));
            }
        }
    }

    fn push(&mut self, target: Target) {
        let v = match target {
            Target::AF | Target::BC | Target::DE | Target::HL => {
                self.registers.combined_register(target)
            }
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::PUSH(target)));
            }
        };

        let mut bus = self.bus.borrow_mut();
        let _ = bus.write(self.sp, (v & 0b11111111) as u8);
        let _ = bus.write(self.sp - 1, ((v & 0b1111111100000000) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(2);
    }

    fn res(&mut self, bit: u8, reg: Target) {
        if reg == Target::HL {
            let mask = !(1 << bit);
            let v = self.registers.combined_register(Target::HL);
            self.registers.set_combined_register(Target::HL, v & mask);
        } else {
            let mask = !(1 << bit);
            match reg {
                Target::A => self.registers.a &= mask,
                Target::B => self.registers.b &= mask,
                Target::C => self.registers.c &= mask,
                Target::D => self.registers.d &= mask,
                Target::E => self.registers.e &= mask,
                Target::H => self.registers.h &= mask,
                Target::L => self.registers.l &= mask,
                _ => {
                    unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::RES(bit, reg)));
                }
            }
        }
    }

    fn ret(&mut self, flag: Flag) -> bool {
        if self.registers.get_flag(flag) {
            self.pc = ((self.bus.borrow().read(self.sp + 1).unwrap() as u16) << 8)
                | (self.bus.borrow().read(self.sp + 2).unwrap() as u16);
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
        let f = |old: u8, cpu: &mut Cpu| {
            let new_carry = old >> 7;
            let old_carry = if cpu.registers.get_flag(Flag::Carry) {
                1
            } else {
                0
            };
            cpu.registers.set_flag_from_u8(Flag::Carry, new_carry);
            (old << 1) | old_carry
        };
        let (old, new) = match reg {
            Target::A => {
                let old = self.registers.a;
                self.registers.a = f(self.registers.a, self);
                (old, self.registers.a)
            }
            Target::B => {
                let old = self.registers.b;
                self.registers.b = f(self.registers.b, self);
                (old, self.registers.b)
            }
            Target::C => {
                let old = self.registers.c;
                self.registers.c = f(self.registers.c, self);
                (old, self.registers.c)
            }
            Target::D => {
                let old = self.registers.d;
                self.registers.d = f(self.registers.d, self);
                (old, self.registers.d)
            }
            Target::E => {
                let old = self.registers.e;
                self.registers.e = f(self.registers.e, self);
                (old, self.registers.e)
            }
            Target::F => {
                let old = self.registers.f;
                self.registers.f = f(self.registers.f, self);
                (old, self.registers.f)
            }
            Target::H => {
                let old = self.registers.h;
                self.registers.h = f(self.registers.h, self);
                (old, self.registers.h)
            }
            Target::L => {
                let old = self.registers.l;
                self.registers.l = f(self.registers.l, self);
                (old, self.registers.l)
            }
            Target::HL => {
                let old = self.registers.combined_register(Target::HL);
                let lower = old & 0b11111111;
                let upper = (old & 0b1111111100000000) >> 7;
                let old_carry = if self.registers.get_flag(Flag::Carry) {
                    1
                } else {
                    0
                };
                let msb_upper = (upper >> 7) as u8;
                let msb_lower = (lower >> 7) as u8;
                let new_upper = (upper << 1) as u8 | old_carry;
                let new_carry = msb_lower as u8;
                let new_lower = (lower << 1) as u8 | msb_upper;
                let new_value = ((new_upper as u16) << 7) + new_lower as u16;
                self.registers.set_combined_register(Target::HL, new_value);
                self.registers.set_flag(Flag::Zero, new_value == 0);
                self.registers.set_flag(Flag::Sub, false);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Carry, new_carry > 0);
                return;
            }
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::RL(reg)));
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
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::RLC(reg)));
            }
        };
    }

    fn rr(&mut self, reg: Target) {
        let f = |old: u8, cpu: &mut Cpu| {
            let new_carry = old & 0b1;
            let old_carry = if cpu.registers.get_flag(Flag::Carry) {
                println!("carry 1");
                1
            } else {
                println!("carry 0");
                0
            };
            let new = (old >> 1) | (old_carry << 7);
            cpu.registers.set_flag_from_u8(Flag::Carry, new_carry);
            new
        };
        let (old, new) = match reg {
            Target::A => {
                let old = self.registers.a;
                self.registers.a = f(self.registers.a, self);
                (old, self.registers.a)
            }
            Target::B => {
                let old = self.registers.b;
                self.registers.b = f(self.registers.b, self);
                (old, self.registers.b)
            }
            Target::C => {
                let old = self.registers.c;
                self.registers.c = f(self.registers.c, self);
                (old, self.registers.c)
            }
            Target::D => {
                let old = self.registers.d;
                self.registers.d = f(self.registers.d, self);
                (old, self.registers.d)
            }
            Target::E => {
                let old = self.registers.e;
                self.registers.e = f(self.registers.e, self);
                (old, self.registers.e)
            }
            Target::F => {
                let old = self.registers.f;
                self.registers.f = f(self.registers.f, self);
                (old, self.registers.f)
            }
            Target::H => {
                let old = self.registers.h;
                self.registers.h = f(self.registers.h, self);
                (old, self.registers.h)
            }
            Target::L => {
                let old = self.registers.l;
                self.registers.l = f(self.registers.l, self);
                (old, self.registers.l)
            }
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::RR(reg)));
            }
        };

        self.set_flags(Instruction::from_opcode(OpCode::RR(reg)).unwrap(), old, new);
    }

    #[allow(unused_assignments)]
    fn rrc(&mut self, reg: Target) {
        let f = |old: u8, cpu: &mut Cpu| {
            let old_carry = if cpu.registers.get_flag(Flag::Carry) {
                1
            } else {
                0
            };
            let new_carry = old & 1;
            cpu.registers.set_flag(Flag::Carry, new_carry == 1);
            let new = (old >> 1) | (old_carry << 7);
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
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::RRC(reg)));
            }
        };
    }

    fn rst(&mut self, address: u16) {
        let _ = self
            .bus
            .borrow_mut()
            .write(self.sp - 1, ((self.pc & 0b1111111100000000) >> 8) as u8);
        let _ = self
            .bus
            .borrow_mut()
            .write(self.sp, (self.pc & 0b11111111) as u8);

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
                .borrow()
                .read(self.registers.combined_register(reg))
                .unwrap(),
            Target::D8 => self.bus.borrow().read(self.pc + 1).unwrap(),
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::SBC(reg)));
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
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::SET(bit, reg)));
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
                unimplemented!("Unimplemented SL, NOT SURE IF THIS INSTRUCTION EXISTS");
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
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::SLA(reg)));
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
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::SRL(reg)));
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
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::SRA(reg)));
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
            Target::HL => self.registers.combined_register(Target::HL) as u8,
            Target::D8 => self.bus.borrow().read(self.pc + 1).unwrap(),
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::SUB(src)));
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
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::SWAP(reg)));
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
                    .borrow()
                    .read(self.registers.combined_register(src))
                    .unwrap()
            }
            Target::D8 => self.registers.a ^= self.bus.borrow().read(self.pc + 1).unwrap(),
            _ => {
                unimplemented!("Unimplemented {}", format!("{:#?}", OpCode::XOR(src)));
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

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{cell::RefCell, rc::Rc};

    use rstest::rstest;

    use crate::{
        consoles::{
            bus::Bus,
            gameboy::{
                cpu::Cpu,
                instruction::Instruction,
                opcode::OpCode::EndOfProgram,
                registers::{Flag, ZERO_BIT_POS},
                target::Target,
            },
            memory::Memory,
            readable::Readable,
            writeable::Writeable,
        },
        utils::conversion::u16_to_u8,
    };

    fn setup() -> Cpu {
        let get_default_value = || Instruction::byte_from_opcode(EndOfProgram).unwrap();
        let memory = Rc::new(RefCell::new(Memory::<u16, u8, u16, 0x10000>::new(
            u16_to_u8,
            Some(Box::new(get_default_value)),
        )));
        let mut bus = Bus::<u16, u8, u16>::new();
        bus.connect_readable(memory.clone());
        bus.connect_writeable(memory);
        let bus = Rc::new(RefCell::new(bus));
        Cpu::new(bus.clone())
    }

    #[rstest]
    #[case(Target::A, true, 3)]
    #[case(Target::A, false, 2)]
    #[case(Target::B, false, 255)]
    #[case(Target::C, true, 0)]
    #[case(Target::C, false, 255)]
    #[case(Target::D, true, 0)]
    #[case(Target::D, false, 255)]
    #[case(Target::E, true, 0)]
    #[case(Target::E, false, 255)]
    #[case(Target::H, true, 0)]
    #[case(Target::H, false, 255)]
    #[case(Target::L, true, 0)]
    #[case(Target::L, false, 255)]
    #[case(Target::HL, true, 0)]
    #[case(Target::HL, false, 255)]
    fn test_adc(#[case] src: Target, #[case] carry: bool, #[case] expected: u8) {
        let mut cpu = setup();

        cpu.registers.set_flag(Flag::Carry, carry);
        cpu.registers.a = 254;
        match src {
            Target::A => cpu.registers.a = 1,
            Target::B => cpu.registers.b = 1,
            Target::C => cpu.registers.c = 1,
            Target::D => cpu.registers.d = 1,
            Target::E => cpu.registers.e = 1,
            Target::H => cpu.registers.h = 1,
            Target::L => cpu.registers.l = 1,
            Target::HL => {
                let _ = cpu.bus.borrow_mut().write(100, 1);
                cpu.registers.set_combined_register(Target::HL, 100);
            }
            _ => panic!("Unsupported register"),
        }
        cpu.adc(src);

        assert_eq!(cpu.registers.a, expected);
    }

    #[rstest]
    #[case(0, Target::B, 5, 5, false, false)]
    #[case(0, Target::C, 5, 5, false, false)]
    #[case(0, Target::D, 5, 5, false, false)]
    #[case(0, Target::E, 5, 5, false, false)]
    #[case(0, Target::H, 5, 5, false, false)]
    #[case(0, Target::L, 5, 5, false, false)]
    #[case(0, Target::HL, 5, 5, false, false)]
    #[case(255, Target::B, 1, 0, true, false)]
    #[case(255, Target::C, 1, 0, true, false)]
    #[case(255, Target::D, 1, 0, true, false)]
    #[case(255, Target::E, 1, 0, true, false)]
    #[case(255, Target::H, 1, 0, true, false)]
    #[case(255, Target::L, 1, 0, true, false)]
    #[case(255, Target::HL, 1, 0, true, false)]
    #[case(0b1111, Target::B, 1, 0b10000, false, true)]
    #[case(0b1111, Target::C, 1, 0b10000, false, true)]
    #[case(0b1111, Target::D, 1, 0b10000, false, true)]
    #[case(0b1111, Target::E, 1, 0b10000, false, true)]
    #[case(0b1111, Target::H, 1, 0b10000, false, true)]
    #[case(0b1111, Target::L, 1, 0b10000, false, true)]
    #[case(0b1111, Target::HL, 1, 0b10000, false, true)]
    fn test_add(
        #[case] val_a: u8,
        #[case] src: Target,
        #[case] val_b: u8,
        #[case] expected: u8,
        #[case] expected_carry_flag: bool,
        #[case] expected_half_carry_flag: bool,
    ) {
        let mut cpu = setup();

        cpu.registers.a = val_a;
        match src {
            Target::B => cpu.registers.b = val_b,
            Target::C => cpu.registers.c = val_b,
            Target::D => cpu.registers.d = val_b,
            Target::E => cpu.registers.e = val_b,
            Target::H => cpu.registers.h = val_b,
            Target::L => cpu.registers.l = val_b,
            Target::HL => {
                let _ = cpu.bus.borrow_mut().write(100, val_b);
                cpu.registers.set_combined_register(Target::HL, 100);
            }
            _ => panic!("Unsupported register"),
        }
        cpu.add(src);
        assert_eq!(cpu.registers.a, expected, "False expected value");
        assert_eq!(
            cpu.registers.get_flag(Flag::Zero),
            expected == 0,
            "False zero flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::Carry),
            expected_carry_flag,
            "False carry flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::HalfCarry),
            expected_half_carry_flag,
            "False half carry flag"
        );
        assert!(!cpu.registers.get_flag(Flag::Sub), "False sub flag");
    }

    #[rstest]
    #[case(Target::BC, 1, 0b10000000, Target::HL, 1, 0b10000000)]
    #[case(Target::DE, 1, 0b10000000, Target::HL, 1, 0b10000000)]
    #[case(Target::HL, 1, 0b10000000, Target::HL, 3, 0b00000000)]
    #[case(Target::SP, 1, 0b10000000, Target::HL, 1, 0b10000000)]
    fn test_add16(
        #[case] src: Target,
        #[case] upper_byte: u8,
        #[case] lower_byte: u8,
        #[case] dst: Target,
        #[case] expeced_upper: u8,
        #[case] expected_lower: u8,
    ) {
        let mut cpu = setup();

        match src {
            Target::BC => {
                cpu.registers.b = upper_byte;
                cpu.registers.c = lower_byte;
            }
            Target::DE => {
                cpu.registers.d = upper_byte;
                cpu.registers.e = lower_byte;
            }
            Target::HL => {
                cpu.registers.h = upper_byte;
                cpu.registers.l = lower_byte;
            }
            Target::SP => cpu.sp = ((upper_byte as u16) << 8) + lower_byte as u16,
            _ => panic!("Unsupported register"),
        }

        cpu.add_16(dst, src);

        match dst {
            Target::HL => {
                assert_eq!(cpu.registers.h, expeced_upper);
                assert_eq!(cpu.registers.l, expected_lower);
            }
            Target::SP => {
                assert_eq!((cpu.sp >> 8) as u8, expeced_upper);
                assert_eq!((cpu.sp & 0xFF) as u8, expected_lower);
            }
            _ => panic!("Unsupported register"),
        }
    }

    #[rstest]
    #[case(3, Target::B, 5, 1, false)]
    #[case(2, Target::B, 5, 0, true)]
    #[case(2, Target::B, 6, 2, false)]
    #[case(3, Target::C, 5, 1, false)]
    #[case(2, Target::C, 5, 0, true)]
    #[case(3, Target::C, 6, 2, false)]
    #[case(3, Target::D, 5, 1, false)]
    #[case(2, Target::D, 5, 0, true)]
    #[case(2, Target::D, 6, 2, false)]
    #[case(3, Target::E, 5, 1, false)]
    #[case(2, Target::E, 5, 0, true)]
    #[case(2, Target::E, 6, 2, false)]
    #[case(3, Target::H, 5, 1, false)]
    #[case(2, Target::H, 5, 0, true)]
    #[case(2, Target::H, 6, 2, false)]
    #[case(3, Target::L, 5, 1, false)]
    #[case(2, Target::L, 5, 0, true)]
    #[case(2, Target::L, 6, 2, false)]
    #[case(3, Target::HL, 5, 1, false)]
    #[case(2, Target::HL, 5, 0, true)]
    #[case(2, Target::HL, 6, 2, false)]
    fn test_and(
        #[case] a_value: u8,
        #[case] src: Target,
        #[case] src_value: u8,
        #[case] expected: u8,
        #[case] expected_zero_flag: bool,
    ) {
        let mut cpu = setup();

        cpu.registers.a = a_value;
        match src {
            Target::B => cpu.registers.b = src_value,
            Target::C => cpu.registers.c = src_value,
            Target::D => cpu.registers.d = src_value,
            Target::E => cpu.registers.e = src_value,
            Target::H => cpu.registers.h = src_value,
            Target::L => cpu.registers.l = src_value,
            Target::HL => {
                let _ = cpu.bus.borrow_mut().write(100, src_value);
                cpu.registers.set_combined_register(Target::HL, 100);
            }
            _ => panic!("Unsupported register"),
        }
        cpu.and(src);

        assert_eq!(cpu.registers.a, expected);
        assert_eq!(cpu.registers.get_flag(Flag::Zero), expected_zero_flag);
        assert!(!cpu.registers.get_flag(Flag::Sub));
        assert!(cpu.registers.get_flag(Flag::HalfCarry));
        assert!(!cpu.registers.get_flag(Flag::Carry));
    }

    #[rstest]
    #[case(0b10000000, 7, Target::A, false, false, false, true)]
    #[case(0b00000000, 7, Target::A, true, false, false, true)]
    #[case(0b10000000, 7, Target::B, false, false, false, true)]
    #[case(0b00000000, 7, Target::B, true, false, false, true)]
    #[case(0b10000000, 7, Target::C, false, false, false, true)]
    #[case(0b00000000, 7, Target::C, true, false, false, true)]
    #[case(0b10000000, 7, Target::D, false, false, false, true)]
    #[case(0b00000000, 7, Target::D, true, false, false, true)]
    #[case(0b10000000, 7, Target::E, false, false, false, true)]
    #[case(0b00000000, 7, Target::E, true, false, false, true)]
    #[case(0b10000000, 7, Target::H, false, false, false, true)]
    #[case(0b00000000, 7, Target::H, true, false, false, true)]
    #[case(0b10000000, 7, Target::L, false, false, false, true)]
    #[case(0b00000000, 7, Target::L, true, false, false, true)]
    #[case(0b10000000, 7, Target::HL, false, false, false, true)]
    #[case(0b00000000, 7, Target::HL, true, false, false, true)]
    fn test_bit(
        #[case] value: u8,
        #[case] bit_pos: u8,
        #[case] src: Target,
        #[case] expected_zero_flag: bool,
        #[case] expected_sub_flag: bool,
        #[case] expected_carry_flag: bool,
        #[case] expected_half_carry_flag: bool,
    ) {
        let mut cpu = setup();

        match src {
            Target::A => cpu.registers.a = value,
            Target::B => cpu.registers.b = value,
            Target::C => cpu.registers.c = value,
            Target::D => cpu.registers.d = value,
            Target::E => cpu.registers.e = value,
            Target::H => cpu.registers.h = value,
            Target::L => cpu.registers.l = value,
            Target::HL => {
                let _ = cpu.bus.borrow_mut().write(100, value);
                cpu.registers.set_combined_register(Target::HL, 100);
            }
            _ => panic!("Unsupported register"),
        }
        cpu.bit(bit_pos, src);

        assert_eq!(cpu.registers.get_flag(Flag::Zero), expected_zero_flag);
        assert_eq!(cpu.registers.get_flag(Flag::Sub), expected_sub_flag);
        assert_eq!(cpu.registers.get_flag(Flag::Carry), expected_carry_flag);
        assert_eq!(
            cpu.registers.get_flag(Flag::HalfCarry),
            expected_half_carry_flag
        );
    }

    #[test]
    fn test_call_and_ret() {
        let mut cpu = setup();

        eprintln!("pc = {}", cpu.pc);
        let address1 = cpu.pc + 10;
        let address2 = cpu.pc;

        let _ = cpu
            .bus
            .borrow_mut()
            .write(cpu.pc + 1, (address1 & 0b11111111) as u8);
        let _ = cpu
            .bus
            .borrow_mut()
            .write(cpu.pc + 2, ((address1 & 0b1111111100000000) >> 8) as u8);
        assert!(cpu.call(Flag::NotZero));
        assert_eq!(cpu.pc, address1);

        assert!(cpu.ret(Flag::NotZero));
        assert_eq!(cpu.pc, address2);
    }

    #[rstest]
    #[case(1, 0b11111110)]
    #[case(2, 0b11111101)]
    #[case(3, 0b11111100)]
    #[case(4, 0b11111011)]
    #[case(8, 0b11110111)]
    #[case(16, 0b11101111)]
    #[case(32, 0b11011111)]
    #[case(64, 0b10111111)]
    #[case(128, 0b01111111)]
    fn test_cpl(#[case] a_value: u8, #[case] expected: u8) {
        let mut cpu = setup();

        cpu.registers.a = a_value;
        cpu.cpl();
        assert_eq!(cpu.registers.a, expected);
        //Zero and Carry flag are not affected, thus will be 0 because they are initialised to 0
        assert!(!cpu.registers.get_flag(Flag::Zero));
        assert!(cpu.registers.get_flag(Flag::Sub));
        assert!(cpu.registers.get_flag(Flag::HalfCarry));
        assert!(!cpu.registers.get_flag(Flag::Carry));
    }

    #[rstest]
    #[case(Target::A, 1, 0, true, true, false, false)]
    #[case(Target::B, 1, 0, true, true, false, false)]
    #[case(Target::C, 1, 0, true, true, false, false)]
    #[case(Target::D, 1, 0, true, true, false, false)]
    #[case(Target::E, 1, 0, true, true, false, false)]
    #[case(Target::H, 1, 0, true, true, false, false)]
    #[case(Target::L, 1, 0, true, true, false, false)]
    #[case(Target::HL, 1, 0, true, true, false, false)]
    #[case(Target::A, 0b10000, 0b1111, false, true, true, false)]
    #[case(Target::B, 0b10000, 0b1111, false, true, true, false)]
    #[case(Target::C, 0b10000, 0b1111, false, true, true, false)]
    #[case(Target::D, 0b10000, 0b1111, false, true, true, false)]
    #[case(Target::E, 0b10000, 0b1111, false, true, true, false)]
    #[case(Target::H, 0b10000, 0b1111, false, true, true, false)]
    #[case(Target::L, 0b10000, 0b1111, false, true, true, false)]
    #[case(Target::HL, 0b10000, 0b1111, false, true, true, false)]
    fn test_dec(
        #[case] reg: Target,
        #[case] reg_value: u8,
        #[case] expected: u8,
        #[case] expected_zero_flag: bool,
        #[case] expected_sub_flag: bool,
        #[case] expected_half_carry_flag: bool,
        #[case] expected_carry_flag: bool,
    ) {
        let mut cpu = setup();
        match reg {
            Target::A => cpu.registers.a = reg_value,
            Target::B => cpu.registers.b = reg_value,
            Target::C => cpu.registers.c = reg_value,
            Target::D => cpu.registers.d = reg_value,
            Target::E => cpu.registers.e = reg_value,
            Target::H => cpu.registers.h = reg_value,
            Target::L => cpu.registers.l = reg_value,
            Target::HL => cpu
                .registers
                .set_combined_register(Target::HL, reg_value as u16),
            _ => panic!("Unsupported register"),
        }
        cpu.dec(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            Target::HL => cpu.registers.combined_register(Target::HL) as u8,
            _ => panic!("Unsupported register"),
        };

        assert_eq!(result, expected, "False result");
        assert_eq!(
            cpu.registers.get_flag(Flag::Zero),
            expected_zero_flag,
            "False zero flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::Sub),
            expected_sub_flag,
            "False sub flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::HalfCarry),
            expected_half_carry_flag,
            "False half carry flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::Carry),
            expected_carry_flag,
            "False carry flag"
        );
    }

    #[rstest]
    #[case(0b100000000, Target::HL, 0b11111111)]
    #[case(0b100000000, Target::BC, 0b11111111)]
    #[case(0b100000000, Target::DE, 0b11111111)]
    #[case(0b100000000, Target::SP, 0b11111111)]
    fn test_dec_16(#[case] reg_value: u16, #[case] reg: Target, #[case] expected: u16) {
        let mut cpu = setup();

        if reg == Target::SP {
            cpu.sp = reg_value;
        } else {
            cpu.registers.set_combined_register(reg, reg_value);
        }
        cpu.dec_16(reg);

        if reg == Target::SP {
            assert_eq!(cpu.sp, expected);
        } else {
            assert_eq!(cpu.registers.combined_register(reg), expected);
        }
    }

    #[rstest]
    #[case(0, Target::A, 1, false, false, false, false)]
    #[case(0, Target::B, 1, false, false, false, false)]
    #[case(0, Target::C, 1, false, false, false, false)]
    #[case(0, Target::D, 1, false, false, false, false)]
    #[case(0, Target::E, 1, false, false, false, false)]
    #[case(0, Target::H, 1, false, false, false, false)]
    #[case(0, Target::L, 1, false, false, false, false)]
    #[case(255, Target::L, 0, true, false, false, false)]
    #[case(255, Target::A, 0, true, false, false, false)]
    #[case(255, Target::B, 0, true, false, false, false)]
    #[case(255, Target::C, 0, true, false, false, false)]
    #[case(255, Target::D, 0, true, false, false, false)]
    #[case(255, Target::E, 0, true, false, false, false)]
    #[case(255, Target::H, 0, true, false, false, false)]
    #[case(255, Target::L, 0, true, false, false, false)]
    #[case(0b1111, Target::A, 0b10000, false, false, true, false)]
    #[case(0b1111, Target::B, 0b10000, false, false, true, false)]
    #[case(0b1111, Target::C, 0b10000, false, false, true, false)]
    #[case(0b1111, Target::D, 0b10000, false, false, true, false)]
    #[case(0b1111, Target::E, 0b10000, false, false, true, false)]
    #[case(0b1111, Target::H, 0b10000, false, false, true, false)]
    fn test_inc(
        #[case] reg_value: u8,
        #[case] reg: Target,
        #[case] expected: u8,
        #[case] expected_zero_flag: bool,
        #[case] expected_sub_flag: bool,
        #[case] expected_half_carry_flag: bool,
        #[case] expected_carry_flag: bool,
    ) {
        let mut cpu = setup();
        match reg {
            Target::A => cpu.registers.a = reg_value,
            Target::B => cpu.registers.b = reg_value,
            Target::C => cpu.registers.c = reg_value,
            Target::D => cpu.registers.d = reg_value,
            Target::E => cpu.registers.e = reg_value,
            Target::H => cpu.registers.h = reg_value,
            Target::L => cpu.registers.l = reg_value,
            _ => panic!("Unsupported register"),
        }
        cpu.inc(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            _ => panic!("Unsupported register"),
        };

        assert_eq!(result, expected, "False result");
        assert_eq!(
            cpu.registers.get_flag(Flag::Zero),
            expected_zero_flag,
            "False zero flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::Sub),
            expected_sub_flag,
            "False sub flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::HalfCarry),
            expected_half_carry_flag,
            "False half carry flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::Carry),
            expected_carry_flag,
            "False carry flag"
        );
    }

    #[rstest]
    #[case(0, Target::HL, 1)]
    #[case(0xFFFF, Target::HL, 0)]
    #[case(0, Target::BC, 1)]
    #[case(0xFFFF, Target::BC, 0)]
    #[case(0, Target::DE, 1)]
    #[case(0xFFFF, Target::DE, 0)]
    #[case(0, Target::SP, 1)]
    #[case(0xFFFF, Target::SP, 0)]
    fn test_inc_16(#[case] reg_value: u16, #[case] reg: Target, #[case] expected: u16) {
        let mut cpu = setup();
        if reg == Target::SP {
            cpu.sp = reg_value;
        } else {
            match reg {
                Target::HL => cpu.registers.set_combined_register(Target::HL, reg_value),
                Target::BC => cpu.registers.set_combined_register(Target::BC, reg_value),
                Target::DE => cpu.registers.set_combined_register(Target::DE, reg_value),
                _ => panic!("Unsupported register"),
            }
        }
        cpu.inc_16(reg);

        let result = if reg == Target::SP {
            cpu.sp
        } else {
            match reg {
                Target::HL => cpu.registers.combined_register(Target::HL),
                Target::BC => cpu.registers.combined_register(Target::BC),
                Target::DE => cpu.registers.combined_register(Target::DE),
                _ => panic!("Unsupported register"),
            }
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_jump() {
        let mut cpu = setup();

        let address = 100_u16;
        let _ = cpu
            .bus
            .borrow_mut()
            .write(cpu.pc + 1, (address & 0xFF) as u8);
        let _ = cpu.bus.borrow_mut().write(cpu.pc + 2, (address >> 8) as u8);
        cpu.jp();

        assert_eq!(cpu.pc, 100);
    }

    #[test]
    fn test_jump_hl() {
        let mut cpu = setup();

        cpu.registers
            .set_combined_register(Target::HL, 0b0000001010001000);

        cpu.jump_hl();

        assert_eq!(cpu.pc, 0b0000001010001000);
    }

    #[rstest]
    #[case(Flag::Zero, true, Flag::Zero)]
    #[case(Flag::Zero, false, Flag::NotZero)]
    #[case(Flag::Carry, true, Flag::Carry)]
    #[case(Flag::Carry, false, Flag::NotCarry)]
    fn test_jump_by_flag(
        #[case] flag_to_set: Flag,
        #[case] flag_value: bool,
        #[case] jump_flag: Flag,
    ) {
        let mut cpu = setup();

        cpu.zero_memory();

        let initial = cpu.pc as u8;

        let _ = cpu.bus.borrow_mut().write(cpu.pc + 1, initial + 255);
        let _ = cpu.bus.borrow_mut().write(cpu.pc + 2, initial);
        cpu.registers.set_flag(flag_to_set, flag_value);
        cpu.jump_by_flag(jump_flag);

        assert!(cpu.pc as u8 == initial + 255);
    }

    #[test]
    fn test_jruc() {
        let mut cpu = setup();

        let initial = cpu.pc as u8;
        cpu.write_to_memory(cpu.pc + 1, initial + 100);
        cpu.jruc();

        assert_eq!(cpu.pc as u8, initial + 100);
    }

    #[rstest]
    #[case(Flag::Zero, true, Flag::Zero)]
    #[case(Flag::Zero, false, Flag::NotZero)]
    #[case(Flag::Carry, true, Flag::Carry)]
    #[case(Flag::Carry, false, Flag::NotCarry)]
    fn test_jr(#[case] flag_to_set: Flag, #[case] flag_value: bool, #[case] jump_flag: Flag) {
        let mut cpu = setup();

        let initial = cpu.pc as u8;
        cpu.write_to_memory(cpu.pc + 1, initial + 100);
        cpu.registers.set_flag(flag_to_set, flag_value);
        cpu.jr(jump_flag);

        assert_eq!(cpu.pc as u8, initial + 100);
    }

    #[rstest]
    #[case(Target::A)]
    #[case(Target::B)]
    #[case(Target::C)]
    #[case(Target::D)]
    #[case(Target::E)]
    #[case(Target::H)]
    #[case(Target::L)]
    #[case(Target::BC)]
    #[case(Target::DE)]
    #[case(Target::HL)]
    fn test_load(#[case] dst: Target) {
        let mut cpu = setup();
        cpu.registers.a = 100;
        cpu.load(dst, Target::A);
        let result = match dst {
            Target::A => cpu.registers.a as u16,
            Target::B => cpu.registers.b as u16,
            Target::C => cpu.registers.c as u16,
            Target::D => cpu.registers.d as u16,
            Target::E => cpu.registers.e as u16,
            Target::H => cpu.registers.h as u16,
            Target::L => cpu.registers.l as u16,
            Target::BC => cpu.registers.combined_register(Target::BC),
            Target::DE => cpu.registers.combined_register(Target::DE),
            Target::HL => cpu.registers.combined_register(Target::HL),
            Target::D8 => {
                unimplemented!();
            }
            Target::A16 => {
                unimplemented!();
            }
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, 100);
    }

    #[rstest]
    #[case(Target::A, 3)]
    #[case(Target::B, 7)]
    #[case(Target::C, 7)]
    #[case(Target::D, 7)]
    #[case(Target::E, 7)]
    #[case(Target::H, 7)]
    #[case(Target::L, 7)]
    #[case(Target::HL, 7)]
    fn test_or(#[case] reg: Target, #[case] expected: u8) {
        let mut cpu = setup();

        cpu.registers.a = 3;
        match reg {
            Target::A => {}
            Target::B => cpu.registers.b = 5,
            Target::C => cpu.registers.c = 5,
            Target::D => cpu.registers.d = 5,
            Target::E => cpu.registers.e = 5,
            Target::H => cpu.registers.h = 5,
            Target::L => cpu.registers.l = 5,
            Target::HL => cpu.registers.set_combined_register(Target::HL, 5),
            _ => panic!("Unsupported register"),
        }
        cpu.or(reg);

        assert_eq!(cpu.registers.a, expected);
    }

    #[test]
    fn test_push_and_pop() {
        let mut cpu = setup();

        cpu.registers
            .set_combined_register(Target::HL, 0b1000100000010001);

        cpu.push(Target::HL);

        assert_eq!(cpu.bus.borrow().read(cpu.sp + 1).unwrap(), 0b10001000);
        assert_eq!(cpu.bus.borrow().read(cpu.sp + 2).unwrap(), 0b00010001);

        cpu.pop(Target::BC);

        assert_eq!(
            cpu.registers.combined_register(Target::BC),
            0b1000100000010001
        );
    }

    #[rstest]
    #[case(Target::B, 5, 5, false, true, false, false)]
    #[case(Target::B, 10, 0, true, true, false, false)]
    #[case(Target::C, 5, 5, false, true, false, false)]
    #[case(Target::C, 10, 0, true, true, false, false)]
    #[case(Target::D, 5, 5, false, true, false, false)]
    #[case(Target::D, 10, 0, true, true, false, false)]
    #[case(Target::E, 5, 5, false, true, false, false)]
    #[case(Target::E, 10, 0, true, true, false, false)]
    #[case(Target::H, 5, 5, false, true, false, false)]
    #[case(Target::H, 10, 0, true, true, false, false)]
    #[case(Target::L, 5, 5, false, true, false, false)]
    #[case(Target::L, 10, 0, true, true, false, false)]
    #[case(Target::HL, 5, 5, false, true, false, false)]
    #[case(Target::HL, 10, 0, true, true, false, false)]
    fn test_sub(
        #[case] reg: Target,
        #[case] reg_value: u8,
        #[case] expected: u8,
        #[case] expected_zero_flag: bool,
        #[case] expected_sub_flag: bool,
        #[case] expected_half_carry_flag: bool,
        #[case] expected_carry_flag: bool,
    ) {
        let mut cpu = setup();

        cpu.registers.a = 10;
        match reg {
            Target::A => {}
            Target::B => cpu.registers.b = reg_value,
            Target::C => cpu.registers.c = reg_value,
            Target::D => cpu.registers.d = reg_value,
            Target::E => cpu.registers.e = reg_value,
            Target::H => cpu.registers.h = reg_value,
            Target::L => cpu.registers.l = reg_value,
            Target::HL => cpu
                .registers
                .set_combined_register(Target::HL, reg_value as u16),
            _ => panic!("Unsupported register"),
        }
        cpu.sub(reg);

        assert_eq!(cpu.registers.a, expected, "False result");
        assert_eq!(
            cpu.registers.get_flag(Flag::Zero),
            expected_zero_flag,
            "False zero flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::Sub),
            expected_sub_flag,
            "False sub flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::HalfCarry),
            expected_half_carry_flag,
            "False half carry flag"
        );
        assert_eq!(
            cpu.registers.get_flag(Flag::Carry),
            expected_carry_flag,
            "False carry flag"
        );
    }

    #[rstest]
    #[case(Flag::Zero)]
    #[case(Flag::Sub)]
    #[case(Flag::HalfCarry)]
    #[case(Flag::Carry)]
    fn test_flags(#[case] flag: Flag) {
        let mut cpu = setup();

        let value = cpu.registers.get_flag(flag);
        cpu.registers.set_flag(flag, !value);
        assert!(value != cpu.registers.get_flag(flag));

        cpu.registers.set_flag(flag, false);
        assert!(!cpu.registers.get_flag(flag));

        cpu.registers.set_flag(flag, true);
        assert!(cpu.registers.get_flag(flag));
    }

    #[rstest]
    #[case(Target::A, 1, 0, 0)]
    #[case(Target::B, 1, 0, 0)]
    #[case(Target::C, 1, 0, 0)]
    #[case(Target::D, 1, 0, 0)]
    #[case(Target::E, 1, 0, 0)]
    #[case(Target::H, 1, 0, 0)]
    #[case(Target::L, 1, 0, 0)]
    #[case(Target::HL, 1, 0, 0)]
    #[case(Target::A, 1, 1, 1)]
    #[case(Target::B, 1, 1, 1)]
    #[case(Target::C, 1, 1, 1)]
    #[case(Target::D, 1, 1, 1)]
    #[case(Target::E, 1, 1, 1)]
    #[case(Target::H, 1, 1, 1)]
    #[case(Target::L, 1, 1, 1)]
    #[case(Target::HL, 1, 1, 1)]
    fn test_res(
        #[case] reg: Target,
        #[case] reg_value: u8,
        #[case] bit_pos: u8,
        #[case] expected: u8,
    ) {
        let mut cpu = setup();

        match reg {
            Target::A => cpu.registers.a = reg_value,
            Target::B => cpu.registers.b = reg_value,
            Target::C => cpu.registers.c = reg_value,
            Target::D => cpu.registers.d = reg_value,
            Target::E => cpu.registers.e = reg_value,
            Target::H => cpu.registers.h = reg_value,
            Target::L => cpu.registers.l = reg_value,
            Target::HL => cpu
                .registers
                .set_combined_register(Target::HL, reg_value as u16),
            _ => panic!("Unsupported register"),
        }
        cpu.res(bit_pos, reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            Target::HL => cpu.registers.combined_register(Target::HL) as u8,
            _ => panic!("Unsupported register"),
        };

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(Target::A, 1, 2, false, false, false, false)]
    #[case(Target::B, 1, 2, false, false, false, false)]
    #[case(Target::C, 1, 2, false, false, false, false)]
    #[case(Target::D, 1, 2, false, false, false, false)]
    #[case(Target::E, 1, 2, false, false, false, false)]
    #[case(Target::H, 1, 2, false, false, false, false)]
    #[case(Target::L, 1, 2, false, false, false, false)]
    #[case(Target::HL, 1, 2, false, false, false, false)]
    #[case(Target::A, 0b10000000, 0, true, false, false, true)]
    #[case(Target::B, 0b10000000, 0, true, false, false, true)]
    #[case(Target::C, 0b10000000, 0, true, false, false, true)]
    #[case(Target::D, 0b10000000, 0, true, false, false, true)]
    #[case(Target::E, 0b10000000, 0, true, false, false, true)]
    #[case(Target::H, 0b10000000, 0, true, false, false, true)]
    #[case(Target::L, 0b10000000, 0, true, false, false, true)]
    #[case(Target::HL, 0b10000000, 0, true, false, false, true)]
    #[case(Target::A, 0b1000, 0b10000, false, false, false, false)]
    #[case(Target::B, 0b1000, 0b10000, false, false, false, false)]
    #[case(Target::C, 0b1000, 0b10000, false, false, false, false)]
    #[case(Target::D, 0b1000, 0b10000, false, false, false, false)]
    #[case(Target::E, 0b1000, 0b10000, false, false, false, false)]
    #[case(Target::H, 0b1000, 0b10000, false, false, false, false)]
    #[case(Target::L, 0b1000, 0b10000, false, false, false, false)]
    #[case(Target::HL, 0b1000, 0b10000, false, false, false, false)]
    fn test_rl(
        #[case] reg: Target,
        #[case] value: u16,
        #[case] expected: u16,
        #[case] expected_zero_flag: bool,
        #[case] expected_sub_flag: bool,
        #[case] expected_half_carry_flag: bool,
        #[case] expected_carry_flag: bool,
    ) {
        let mut cpu = setup();

        match reg {
            Target::A => cpu.registers.a = value as u8,
            Target::B => cpu.registers.b = value as u8,
            Target::C => cpu.registers.c = value as u8,
            Target::D => cpu.registers.d = value as u8,
            Target::E => cpu.registers.e = value as u8,
            Target::H => cpu.registers.h = value as u8,
            Target::L => cpu.registers.l = value as u8,
            Target::HL => cpu.registers.set_combined_register(Target::HL, value),
            _ => panic!("Unsupported register"),
        }
        cpu.rl(reg);

        let result = match reg {
            Target::A => cpu.registers.a as u16,
            Target::B => cpu.registers.b as u16,
            Target::C => cpu.registers.c as u16,
            Target::D => cpu.registers.d as u16,
            Target::E => cpu.registers.e as u16,
            Target::H => cpu.registers.h as u16,
            Target::L => cpu.registers.l as u16,
            Target::HL => cpu.registers.combined_register(Target::HL),
            _ => panic!("Unsupported register"),
        };

        assert_eq!(result, expected);
        assert_eq!(cpu.registers.get_flag(Flag::Zero), expected_zero_flag);
        assert_eq!(cpu.registers.get_flag(Flag::Sub), expected_sub_flag);
        assert_eq!(
            cpu.registers.get_flag(Flag::HalfCarry),
            expected_half_carry_flag
        );
        assert_eq!(cpu.registers.get_flag(Flag::Carry), expected_carry_flag);
    }

    #[test]
    fn test_rla() {
        let mut cpu = setup();

        cpu.registers.a = 0b10000000;
        cpu.rla();

        assert!(cpu.registers.get_flag(Flag::Carry));
        assert_eq!(cpu.registers.a, 0);

        cpu.rla();

        assert!(!cpu.registers.get_flag(Flag::Carry));
        assert_eq!(cpu.registers.a, 1);
    }

    #[test]
    fn test_rlc() {
        let mut cpu = setup();

        cpu.registers.set_flag(Flag::Carry, true);
        cpu.registers.a = 1;
        cpu.rlc(Target::A);

        assert_eq!(cpu.registers.a, 2);
        assert!(!cpu.registers.get_flag(Flag::Carry));

        cpu.registers.a = 0b10000000;
        cpu.rlc(Target::A);

        assert_eq!(cpu.registers.a, 1);
        assert!(cpu.registers.get_flag(Flag::Carry));
    }

    #[rstest]
    #[case(Target::A, 1, 0, true, 2, 0b10000001, false)]
    #[case(Target::B, 1, 0, true, 2, 0b10000001, false)]
    #[case(Target::C, 1, 0, true, 2, 0b10000001, false)]
    #[case(Target::D, 1, 0, true, 2, 0b10000001, false)]
    #[case(Target::E, 1, 0, true, 2, 0b10000001, false)]
    #[case(Target::H, 1, 0, true, 2, 0b10000001, false)]
    #[case(Target::L, 1, 0, true, 2, 0b10000001, false)]
    fn test_rr(
        #[case] reg: Target,
        #[case] value1: u8,
        #[case] expected1: u8,
        #[case] expected_carry_flag1: bool,
        #[case] value2: u8,
        #[case] expected2: u8,
        #[case] expected_carry_flag2: bool,
    ) {
        let mut cpu = setup();

        match reg {
            Target::A => cpu.registers.a = value1,
            Target::B => cpu.registers.b = value1,
            Target::C => cpu.registers.c = value1,
            Target::D => cpu.registers.d = value1,
            Target::E => cpu.registers.e = value1,
            Target::H => cpu.registers.h = value1,
            Target::L => cpu.registers.l = value1,
            Target::HL => cpu
                .registers
                .set_combined_register(Target::HL, value1 as u16),
            _ => panic!("Unsupported register"),
        }
        cpu.rr(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            Target::HL => cpu.registers.combined_register(Target::HL) as u8,
            _ => panic!("Unsupported register"),
        };

        assert_eq!(result, expected1);
        assert_eq!(cpu.registers.get_flag(Flag::Carry), expected_carry_flag1);

        match reg {
            Target::A => cpu.registers.a = value2,
            Target::B => cpu.registers.b = value2,
            Target::C => cpu.registers.c = value2,
            Target::D => cpu.registers.d = value2,
            Target::E => cpu.registers.e = value2,
            Target::H => cpu.registers.h = value2,
            Target::L => cpu.registers.l = value2,
            Target::HL => cpu
                .registers
                .set_combined_register(Target::HL, value1 as u16),
            _ => panic!("Unsupported register"),
        }
        cpu.rr(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            Target::HL => cpu.registers.combined_register(Target::HL) as u8,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, expected2);
        assert_eq!(cpu.registers.get_flag(Flag::Carry), expected_carry_flag2);
    }

    #[test]
    fn test_rra() {
        let mut cpu = setup();

        cpu.registers.a = 1;
        cpu.rra();

        assert!(cpu.registers.get_flag(Flag::Carry));
        assert_eq!(cpu.registers.a, 0);

        cpu.rra();

        assert!(!cpu.registers.get_flag(Flag::Carry));
        assert_eq!(cpu.registers.a, 0b10000000);
    }

    #[rstest]
    #[case(Target::A, 1, true, 0b10000000, true, 0b11000000, false)]
    #[case(Target::B, 1, true, 0b10000000, true, 0b11000000, false)]
    #[case(Target::C, 1, true, 0b10000000, true, 0b11000000, false)]
    #[case(Target::D, 1, true, 0b10000000, true, 0b11000000, false)]
    #[case(Target::E, 1, true, 0b10000000, true, 0b11000000, false)]
    #[case(Target::H, 1, true, 0b10000000, true, 0b11000000, false)]
    #[case(Target::L, 1, true, 0b10000000, true, 0b11000000, false)]
    fn test_rrc(
        #[case] reg: Target,
        #[case] value1: u8,
        #[case] flag: bool,
        #[case] expected1: u8,
        #[case] expected_flag1: bool,
        #[case] expected2: u8,
        #[case] expected_flag2: bool,
    ) {
        let mut cpu = setup();

        cpu.registers.set_flag(Flag::Carry, flag);
        match reg {
            Target::A => cpu.registers.a = value1,
            Target::B => cpu.registers.b = value1,
            Target::C => cpu.registers.c = value1,
            Target::D => cpu.registers.d = value1,
            Target::E => cpu.registers.e = value1,
            Target::H => cpu.registers.h = value1,
            Target::L => cpu.registers.l = value1,
            Target::HL => cpu
                .registers
                .set_combined_register(Target::HL, value1 as u16),
            _ => panic!("Unsupported register"),
        }
        cpu.rrc(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            Target::HL => cpu.registers.combined_register(Target::HL) as u8,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, expected1);
        assert_eq!(cpu.registers.get_flag(Flag::Carry), expected_flag1);

        cpu.rrc(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            Target::HL => cpu.registers.combined_register(Target::HL) as u8,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, expected2);
        assert_eq!(cpu.registers.get_flag(Flag::Carry), expected_flag2);
    }

    #[test]
    fn test_rrca() {
        let mut cpu = setup();

        cpu.registers.a = 1;
        cpu.rrca();

        assert!(cpu.registers.get_flag(Flag::Carry));
        assert_eq!(cpu.registers.a, (1 << ZERO_BIT_POS));
    }

    #[rstest]
    #[case(Target::B, true, 1, 0xFF)]
    #[case(Target::B, false, 1, 0)]
    #[case(Target::C, true, 1, 0xFF)]
    #[case(Target::C, false, 1, 0)]
    #[case(Target::D, true, 1, 0xFF)]
    #[case(Target::D, false, 1, 0)]
    #[case(Target::E, true, 1, 0xFF)]
    #[case(Target::E, false, 1, 0)]
    #[case(Target::H, true, 1, 0xFF)]
    #[case(Target::H, false, 1, 0)]
    #[case(Target::L, true, 1, 0xFF)]
    #[case(Target::L, false, 1, 0)]
    fn test_sbc(#[case] reg: Target, #[case] flag: bool, #[case] value: u8, #[case] expected: u8) {
        let mut cpu = setup();

        cpu.registers.set_flag(Flag::Carry, flag);
        cpu.registers.a = 1;
        match reg {
            Target::A => {}
            Target::B => cpu.registers.b = value,
            Target::C => cpu.registers.c = value,
            Target::D => cpu.registers.d = value,
            Target::E => cpu.registers.e = value,
            Target::H => cpu.registers.h = value,
            Target::L => cpu.registers.l = value,
            _ => panic!("Unsupported register"),
        }
        cpu.sbc(reg);

        assert_eq!(cpu.registers.a, expected);
    }

    #[rstest]
    #[case(Target::A, 7)]
    #[case(Target::B, 7)]
    #[case(Target::C, 7)]
    #[case(Target::D, 7)]
    #[case(Target::E, 7)]
    #[case(Target::H, 7)]
    #[case(Target::L, 7)]
    fn test_set(#[case] reg: Target, #[case] bit_pos: u8) {
        let mut cpu = setup();

        cpu.set(bit_pos, reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, 128);
    }

    #[rstest]
    #[case(Target::A)]
    #[case(Target::B)]
    #[case(Target::C)]
    #[case(Target::D)]
    #[case(Target::E)]
    #[case(Target::H)]
    #[case(Target::L)]
    fn test_sl(#[case] reg: Target) {
        let mut cpu = setup();

        match reg {
            Target::A => cpu.registers.a = 1,
            Target::B => cpu.registers.b = 1,
            Target::C => cpu.registers.c = 1,
            Target::D => cpu.registers.d = 1,
            Target::E => cpu.registers.e = 1,
            Target::H => cpu.registers.h = 1,
            Target::L => cpu.registers.l = 1,
            _ => panic!("Unsupported register"),
        }
        cpu.sl(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, 2);

        match reg {
            Target::A => cpu.registers.a = 0b10000000,
            Target::B => cpu.registers.b = 0b10000000,
            Target::C => cpu.registers.c = 0b10000000,
            Target::D => cpu.registers.d = 0b10000000,
            Target::E => cpu.registers.e = 0b10000000,
            Target::H => cpu.registers.h = 0b10000000,
            Target::L => cpu.registers.l = 0b10000000,
            _ => panic!("Unsupported register"),
        }
        cpu.sl(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, 0);
    }

    #[test]
    fn test_sla() {
        let mut cpu = setup();

        cpu.registers.a = 1;
        cpu.sla(Target::A);

        assert_eq!(cpu.registers.a, 2);
    }

    #[rstest]
    #[case(Target::A)]
    #[case(Target::B)]
    #[case(Target::C)]
    #[case(Target::D)]
    #[case(Target::E)]
    #[case(Target::H)]
    #[case(Target::L)]
    fn test_sr(#[case] reg: Target) {
        let mut cpu = setup();

        match reg {
            Target::A => cpu.registers.a = 2,
            Target::B => cpu.registers.b = 2,
            Target::C => cpu.registers.c = 2,
            Target::D => cpu.registers.d = 2,
            Target::E => cpu.registers.e = 2,
            Target::H => cpu.registers.h = 2,
            Target::L => cpu.registers.l = 2,
            _ => panic!("Unsupported register"),
        }
        cpu.srl(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, 1);

        cpu.srl(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, 0);
    }

    #[test]
    fn test_sra() {
        let mut cpu = setup();

        cpu.registers.a = 0b10000000;
        cpu.sra(Target::A);

        assert_eq!(cpu.registers.a, 0b11000000);
    }

    #[rstest]
    #[case(Target::A)]
    #[case(Target::B)]
    #[case(Target::C)]
    #[case(Target::D)]
    #[case(Target::E)]
    #[case(Target::H)]
    #[case(Target::L)]
    fn test_swap(#[case] reg: Target) {
        let mut cpu = setup();

        match reg {
            Target::A => cpu.registers.a = 128 + 4,
            Target::B => cpu.registers.b = 128 + 4,
            Target::C => cpu.registers.c = 128 + 4,
            Target::D => cpu.registers.d = 128 + 4,
            Target::E => cpu.registers.e = 128 + 4,
            Target::H => cpu.registers.h = 128 + 4,
            Target::L => cpu.registers.l = 128 + 4,
            _ => panic!("Unsupported register"),
        }
        cpu.swap(reg);

        let result = match reg {
            Target::A => cpu.registers.a,
            Target::B => cpu.registers.b,
            Target::C => cpu.registers.c,
            Target::D => cpu.registers.d,
            Target::E => cpu.registers.e,
            Target::H => cpu.registers.h,
            Target::L => cpu.registers.l,
            _ => panic!("Unsupported register"),
        };
        assert_eq!(result, 72);
    }

    #[rstest]
    #[case(Target::A, 0)]
    #[case(Target::B, 6)]
    #[case(Target::C, 6)]
    #[case(Target::D, 6)]
    #[case(Target::E, 6)]
    #[case(Target::H, 6)]
    #[case(Target::L, 6)]
    fn test_xor(#[case] reg: Target, #[case] expected: u8) {
        let mut cpu = setup();

        cpu.registers.a = 3;
        match reg {
            Target::A => {}
            Target::B => cpu.registers.b = 5,
            Target::C => cpu.registers.c = 5,
            Target::D => cpu.registers.d = 5,
            Target::E => cpu.registers.e = 5,
            Target::H => cpu.registers.h = 5,
            Target::L => cpu.registers.l = 5,
            _ => panic!("Unsupported register"),
        }
        cpu.xor(reg);

        assert_eq!(cpu.registers.a, expected);
    }
}
