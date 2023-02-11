use crate::instruction::Target;
use crate::Flag;
use crate::Instruction;
use crate::Memory;
use crate::OpCode;
use crate::Registers;

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    memory: Memory,
    pc: u16,
    sp: u16,
    addr: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            memory: Memory::new(),
            pc: 0,
            sp: 0,
            addr: 0,
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

        if instruction_byte == Instruction::instruction_byte_from_opcode(OpCode::EndOfProgram) {
            return false;
        }

        let _pc: u16 = if let Some(instruction) = Instruction::from_byte(instruction_byte) {
            self.execute(instruction)
        } else {
            panic!(
                "Uknown instruction: 0x{:x} @ address {}",
                instruction_byte, self.pc
            );
        };
        self.pc = _pc;
        true
    }

    pub fn execute(&mut self, instruction: &Instruction) -> u16 {
        let mut pc_increment = instruction.length as u16;

        match instruction.opcode.to_owned() {
            OpCode::LD(target) => {
                self.registers
                    .load(target, self.memory.read_byte(self.pc + 1));
            }
            OpCode::LDR(target, src) => match src {
                Target::D8 => self
                    .registers
                    .load(target, self.memory.read_byte(self.pc + 1)),
                _ => {
                    self.registers.load_from_register(target, src);
                }
            },
            OpCode::ADD(target) => {
                self.registers.add(target);
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
            OpCode::RRC(target) => {
                self.registers.rotate_right(target);
            }
            OpCode::RLC(target) => {
                self.registers.rotate_right(target);
            }
            OpCode::SRA(target) => {
                self.registers.shift_right(target);
            }
            OpCode::SLA(target) => {
                self.registers.shift_left(target);
            }
            OpCode::CPL => {
                todo!();
            }
            OpCode::BIT(target, bit) => {
                let _b = self.registers.get_bit(target, &bit);
                todo!();
            }
            OpCode::RESET(target, bit) => {
                self.registers.set_bit(target, &bit, 0);
            }
            OpCode::SET(target, bit) => {
                self.registers.set_bit(target, &bit, 1);
            }
            OpCode::SWAP(target) => {
                self.registers.swap(target);
            }
            OpCode::JUMP(flag) => {
                let value = self.memory.read_byte(self.pc + 1);
                match flag {
                    Flag::Zero => {
                        if self.registers.get_flag(Flag::Zero) {
                            pc_increment = self.pc.wrapping_add(value as u16);
                        }
                    }
                    Flag::Carry => {
                        if self.registers.get_flag(Flag::Carry) {
                            pc_increment = self.pc.wrapping_add(value as u16);
                        }
                    }
                    Flag::NotCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            pc_increment = self.pc.wrapping_add(value as u16);
                        }
                    }
                    Flag::NotZero => {
                        if !self.registers.get_flag(Flag::Carry) {
                            pc_increment = self.pc.wrapping_add(value as u16);
                        }
                    }
                    Flag::HalfCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            pc_increment = self.pc.wrapping_add(value as u16);
                        }
                    }
                    Flag::NotHalfCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            pc_increment = self.pc.wrapping_add(value as u16);
                        }
                    }

                    _ => {}
                };
            }
            OpCode::CALL(flag) => match flag {
                Flag::Zero => if self.registers.get_flag(flag) {},
                _ => {}
            },
            _ => {
                panic!("Unimplemented");
            }
        }

        self.pc.wrapping_add(pc_increment)
    }

    pub fn add(&mut self, src: Target) {
        match src {
            Target::A => self.registers.a = self.registers.a.wrapping_add(self.registers.a),
            Target::B => self.registers.a = self.registers.a.wrapping_add(self.registers.b),
            Target::C => self.registers.a = self.registers.a.wrapping_add(self.registers.c),
            Target::D => self.registers.a = self.registers.a.wrapping_add(self.registers.d),
            Target::E => self.registers.a = self.registers.a.wrapping_add(self.registers.e),
            Target::F => self.registers.a = self.registers.a.wrapping_add(self.registers.f),
            Target::G => self.registers.a = self.registers.a.wrapping_add(self.registers.g),
            Target::H => self.registers.a = self.registers.a.wrapping_add(self.registers.h),
            _ => {
                panic!("Invalid Target");
            }
        }
    }

    pub fn add_hl(&mut self, src: Target) {
        match src {
            Target::A => self.registers.hl = self.registers.hl.wrapping_add(self.registers.a),
            Target::B => self.registers.hl = self.registers.hl.wrapping_add(self.registers.b),
            Target::C => self.registers.hl = self.registers.hl.wrapping_add(self.registers.c),
            Target::D => self.registers.hl = self.registers.hl.wrapping_add(self.registers.d),
            Target::E => self.registers.hl = self.registers.hl.wrapping_add(self.registers.e),
            Target::F => self.registers.hl = self.registers.hl.wrapping_add(self.registers.f),
            Target::G => self.registers.hl = self.registers.hl.wrapping_add(self.registers.g),
            Target::H => self.registers.hl = self.registers.hl.wrapping_add(self.registers.h),
            _ => {
                panic!("Unimplemented")
            }
        }
    }

    pub fn sub(&mut self, src: Target) {
        match src {
            Target::A => self.registers.a = self.registers.a.wrapping_sub(self.registers.a),
            Target::B => self.registers.a = self.registers.a.wrapping_sub(self.registers.b),
            Target::C => self.registers.a = self.registers.a.wrapping_sub(self.registers.c),
            Target::D => self.registers.a = self.registers.a.wrapping_sub(self.registers.d),
            Target::E => self.registers.a = self.registers.a.wrapping_sub(self.registers.e),
            Target::F => self.registers.a = self.registers.a.wrapping_sub(self.registers.f),
            Target::G => self.registers.a = self.registers.a.wrapping_sub(self.registers.g),
            Target::H => self.registers.a = self.registers.a.wrapping_sub(self.registers.h),
            _ => {
                panic!("Invalid Target");
            }
        }
    }

    pub fn inc(&mut self, target: Target) {
        match target {
            Target::A => self.registers.a = self.registers.a.wrapping_add(1),
            Target::B => self.registers.b = self.registers.b.wrapping_add(1),
            Target::C => self.registers.c = self.registers.c.wrapping_add(1),
            Target::D => self.registers.d = self.registers.d.wrapping_add(1),
            Target::E => self.registers.e = self.registers.e.wrapping_add(1),
            Target::F => self.registers.f = self.registers.f.wrapping_add(1),
            Target::G => self.registers.g = self.registers.g.wrapping_add(1),
            Target::H => self.registers.h = self.registers.h.wrapping_add(1),
            Target::HL => todo!(),
            Target::HLP => todo!(),
            Target::HLM => todo!(),
            Target::BC => {
                self.registers.b = self.registers.b.wrapping_add(1);
                self.registers.c = self.registers.c.wrapping_add(1);
            }
            Target::DE => {
                self.registers.d = self.registers.d.wrapping_add(1);
                self.registers.e = self.registers.e.wrapping_add(1);
            }
            _ => {
                panic!("Unimplemented")
            }
        }
    }

    pub fn dec(&mut self, target: Target) {
        match target {
            Target::A => self.registers.a = self.registers.a.wrapping_sub(1),
            Target::B => self.registers.b = self.registers.b.wrapping_sub(1),
            Target::C => self.registers.c = self.registers.c.wrapping_sub(1),
            Target::D => self.registers.d = self.registers.d.wrapping_sub(1),
            Target::E => self.registers.e = self.registers.e.wrapping_sub(1),
            Target::F => self.registers.f = self.registers.f.wrapping_sub(1),
            Target::G => self.registers.g = self.registers.g.wrapping_sub(1),
            Target::H => self.registers.h = self.registers.h.wrapping_sub(1),
            Target::HL => todo!(),
            Target::HLP => todo!(),
            Target::HLM => todo!(),
            Target::BC => {
                self.registers.b = self.registers.b.wrapping_sub(1);
                self.registers.c = self.registers.c.wrapping_sub(1);
            }
            Target::DE => {
                self.registers.d = self.registers.d.wrapping_sub(1);
                self.registers.e = self.registers.e.wrapping_sub(1);
            }
            _ => {
                panic!("Unimplemented")
            }
        }
    }

    pub fn and(&mut self, src: Target) {
        match src {
            Target::A => self.registers.a = self.registers.a & self.registers.a,
            Target::B => self.registers.a = self.registers.a & self.registers.b,
            Target::C => self.registers.a = self.registers.a & self.registers.c,
            Target::D => self.registers.a = self.registers.a & self.registers.d,
            Target::E => self.registers.a = self.registers.a & self.registers.e,
            Target::F => self.registers.a = self.registers.a & self.registers.f,
            Target::G => self.registers.a = self.registers.a & self.registers.g,
            Target::H => self.registers.a = self.registers.a & self.registers.h,
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn or(&mut self, src: Target) {
        match src {
            Target::A => self.registers.a = self.registers.a | self.registers.a,
            Target::B => self.registers.a = self.registers.a | self.registers.b,
            Target::C => self.registers.a = self.registers.a | self.registers.c,
            Target::D => self.registers.a = self.registers.a | self.registers.d,
            Target::E => self.registers.a = self.registers.a | self.registers.e,
            Target::F => self.registers.a = self.registers.a | self.registers.f,
            Target::G => self.registers.a = self.registers.a | self.registers.g,
            Target::H => self.registers.a = self.registers.a | self.registers.h,
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn xor(&mut self, src: Target) {
        match src {
            Target::A => self.registers.a = self.registers.a ^ self.registers.a,
            Target::B => self.registers.a = self.registers.a ^ self.registers.b,
            Target::C => self.registers.a = self.registers.a ^ self.registers.c,
            Target::D => self.registers.a = self.registers.a ^ self.registers.d,
            Target::E => self.registers.a = self.registers.a ^ self.registers.e,
            Target::F => self.registers.a = self.registers.a ^ self.registers.f,
            Target::G => self.registers.a = self.registers.a ^ self.registers.g,
            Target::H => self.registers.a = self.registers.a ^ self.registers.h,
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn set_a(&mut self, value: u8) {
        self.registers.a = value;
    }

    pub fn set_b(&mut self, value: u8) {
        self.registers.b = value;
    }

    pub fn print_registers(&self) {
        println!(
            "Registers {{\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n}}",
            self.registers.a,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.f,
            self.registers.g,
            self.registers.h
        );
    }
}

#[test]
fn test_arithmetic() {
    let mut cpu = Cpu::new();

    cpu.registers.b = 5;

    let mut i = Instruction::from_byte(0x80).unwrap();

    cpu.execute(&i);

    assert!(cpu.registers.a == 5);

    i = Instruction::from_byte(0x80).unwrap();
    cpu.registers.a = 255;
    cpu.registers.b = 1;

    cpu.execute(&i);

    assert!(cpu.registers.a == 0);

    i = Instruction::from_byte(0x90).unwrap();
    cpu.execute(&i);

    assert!(cpu.registers.a == 255);
}

#[test]
fn test_increment() {
    let mut cpu = Cpu::new();

    let mut i = Instruction::from_byte(0x3C).unwrap();

    cpu.execute(&i);

    assert!(cpu.registers.a == 1);

    i = Instruction::from_byte(0x3D).unwrap();
    cpu.execute(&i);

    assert!(cpu.registers.a == 0);

    i = Instruction::from_byte(0x03D).unwrap();
    cpu.execute(&i);

    assert!(cpu.registers.a == 255);
}

#[test]
fn test_bitwise() {
    let mut cpu = Cpu::new();

    let mut i = Instruction::from_byte(0xA0).unwrap();
    cpu.registers.a = 5;
    cpu.registers.b = 3;

    cpu.execute(&i);

    assert!(cpu.registers.a == 1);

    i = Instruction::from_byte(0xB0).unwrap();
    cpu.registers.a = 5;
    cpu.registers.b = 2;

    cpu.execute(&i);

    assert!(cpu.registers.a == 7);

    i = Instruction::from_byte(0xA8).unwrap();
    cpu.registers.a = 5;
    cpu.registers.b = 3;

    cpu.execute(&i);

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
