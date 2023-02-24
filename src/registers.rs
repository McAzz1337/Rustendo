use crate::instruction::Target;
use crate::utils;

use lmg_offset::Offset;

pub const ZERO_BIT_POS: u8 = 7;
pub const CARRY_BIT_POS: u8 = 4;
pub const HALF_CARRY_BIT_POS: u8 = 5;
pub const SUB_BIT_POS: u8 = 6;

#[derive(Offset, Debug)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub hl: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Zero,
    Carry,
    HalfCarry,
    Sub,
    NotZero,
    NotCarry,
    NotHalfCarry,
    NotSub,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            l: 0,
            h: 0,
            hl: 0,
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Zero => {
                return (self.f >> ZERO_BIT_POS) == 1;
            }
            Flag::Carry => {
                return (self.f & (1 << CARRY_BIT_POS)) >= 1;
            }
            Flag::HalfCarry => {
                return (self.f & (1 << HALF_CARRY_BIT_POS)) >= 1;
            }
            Flag::Sub => {
                return (self.f & (1 << SUB_BIT_POS)) >= 1;
            }
            Flag::NotZero => {
                return (self.f >> ZERO_BIT_POS) == 0;
            }
            Flag::NotCarry => {
                return (self.f & (1 << CARRY_BIT_POS)) == 0;
            }
            Flag::NotHalfCarry => {
                return (self.f & (1 << HALF_CARRY_BIT_POS)) == 0;
            }
            Flag::NotSub => {
                return (self.f & (1 << SUB_BIT_POS)) == 0;
            }
        }
    }

    pub fn filter_flag(&self, flag: Flag) -> u8 {
        match flag {
            Flag::Zero => return (self.f & (1 << ZERO_BIT_POS)) >> ZERO_BIT_POS,
            Flag::Carry => return (self.f & (1 << CARRY_BIT_POS)) >> CARRY_BIT_POS,
            Flag::HalfCarry => return (self.f & (1 << HALF_CARRY_BIT_POS)) >> HALF_CARRY_BIT_POS,
            Flag::Sub => return (self.f & (1 << SUB_BIT_POS)) >> SUB_BIT_POS,
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn set_flag(&mut self, flag: Flag, v: bool) {
        let mut mask;

        match flag {
            Flag::Zero => {
                mask = 1 << ZERO_BIT_POS;
            }
            Flag::Carry => {
                mask = 1 << CARRY_BIT_POS;
            }
            Flag::HalfCarry => {
                mask = 1 << HALF_CARRY_BIT_POS;
            }
            Flag::Sub => {
                mask = 1 << SUB_BIT_POS;
            }
            _ => {
                panic!("Invalid flag as Input!");
            }
        }

        if !v {
            mask = !mask;
            self.f = self.f & mask;
        } else {
            self.f = self.f | mask;
        }
    }

    pub fn set_flag_from_u8(&mut self, flag: Flag, bit: u8) {
        if bit == 0 {
            self.set_flag(flag, false);
        } else {
            self.set_flag(flag, true);
        }
    }

    pub fn rotate_right(&mut self, reg: Target) {
        match reg {
            Target::A => {
                let x = (self.a & 1) << 7;
                self.a = self.a >> 1;
                self.a = self.a | x;
            }
            Target::B => {
                let x = (self.b & 1) << 7;
                self.b = self.b >> 1;
                self.b = self.b | x;
            }
            Target::C => {
                let x = (self.c & 1) << 7;
                self.c = self.c >> 1;
                self.c = self.c | x;
            }
            Target::D => {
                let x = (self.d & 1) << 7;
                self.d = self.d >> 1;
                self.d = self.d | x;
            }
            Target::E => {
                let x = (self.e & 1) << 7;
                self.e = self.e >> 1;
                self.e = self.e | x;
            }
            Target::F => {
                let x = (self.f & 1) << 7;
                self.f = self.f >> 1;
                self.f = self.f | x;
            }
            Target::L => {
                let x = (self.l & 1) << 7;
                self.l = self.l >> 1;
                self.l = self.l | x;
            }
            Target::H => {
                let x = (self.h & 1) << 7;
                self.h = self.h >> 1;
                self.h = self.h | x;
            }
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn rotate_left(&mut self, reg: Target) {
        match reg {
            Target::A => {
                let x = (self.a & 0b10000000) >> 7;
                self.a = self.a << 1;
                self.a = self.a | x;
            }
            Target::B => {
                let x = (self.b & 0b10000000) >> 7;
                self.b = self.b << 1;
                self.b = self.b | x;
            }
            Target::C => {
                let x = (self.c & 0b10000000) >> 7;
                self.c = self.c << 1;
                self.c = self.c | x;
            }
            Target::D => {
                let x = (self.d & 0b10000000) >> 7;
                self.d = self.d << 1;
                self.d = self.d | x;
            }
            Target::E => {
                let x = (self.e & 0b10000000) >> 7;
                self.e = self.e << 1;
                self.e = self.e | x;
            }
            Target::F => {
                let x = (self.f & 0b10000000) >> 7;
                self.f = self.f << 1;
                self.f = self.f | x;
            }
            Target::L => {
                let x = (self.l & 0b10000000) >> 7;
                self.l = self.l << 1;
                self.l = self.l | x;
            }
            Target::H => {
                let x = (self.h & 0b10000000) >> 7;
                self.h = self.h << 1;
                self.h = self.h | x;
            }
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn shift_right(&mut self, reg: Target) {
        match reg {
            Target::A => {
                self.a = self.a >> 1;
            }
            Target::B => {
                self.b = self.b >> 1;
            }
            Target::C => {
                self.c = self.c >> 1;
            }
            Target::D => {
                self.d = self.d >> 1;
            }
            Target::E => {
                self.e = self.e >> 1;
            }
            Target::F => {
                self.f = self.f >> 1;
            }
            Target::L => {
                self.l = self.l >> 1;
            }
            Target::H => {
                self.h = self.h >> 1;
            }
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn shift_left(&mut self, reg: Target) {
        match reg {
            Target::A => {
                self.a = self.a << 1;
            }
            Target::B => {
                self.b = self.b << 1;
            }
            Target::C => {
                self.c = self.c << 1;
            }
            Target::D => {
                self.d = self.d << 1;
            }
            Target::E => {
                self.e = self.e << 1;
            }
            Target::F => {
                self.f = self.f << 1;
            }
            Target::L => {
                self.l = self.l << 1;
            }
            Target::H => {
                self.h = self.h << 1;
            }
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn get_bit(&self, reg: Target, bit: &u32) -> bool {
        let mask = 1 << bit;
        match reg {
            Target::A => return (self.a & mask) >= 1,
            Target::B => return (self.b & mask) >= 1,
            Target::C => return (self.c & mask) >= 1,
            Target::D => return (self.d & mask) >= 1,
            Target::E => return (self.e & mask) >= 1,
            Target::F => return (self.f & mask) >= 1,
            Target::L => return (self.l & mask) >= 1,
            Target::H => return (self.h & mask) >= 1,
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn set_bit(&mut self, reg: Target, bit: &u32, v: u8) {
        if v == 1 {
            let mask = v << bit;
            match reg {
                Target::A => self.a = self.a | mask,
                Target::B => self.b = self.b | mask,
                Target::C => self.c = self.c | mask,
                Target::D => self.d = self.d | mask,
                Target::E => self.e = self.e | mask,
                Target::F => self.f = self.f | mask,
                Target::L => self.l = self.l | mask,
                Target::H => self.h = self.h | mask,
                _ => {
                    panic!("Unimplemented");
                }
            }
        } else {
            let mut mask = 0b1111111;
            mask = mask ^ (1 << bit);
            match reg {
                Target::A => self.a = self.a & mask,
                Target::B => self.b = self.b & mask,
                Target::C => self.c = self.c & mask,
                Target::D => self.d = self.d & mask,
                Target::E => self.e = self.e & mask,
                Target::F => self.f = self.f & mask,
                Target::L => self.l = self.l & mask,
                Target::H => self.h = self.h & mask,
                _ => {
                    panic!("Unimplemented");
                }
            }
        }
    }

    pub fn register_as_bit_string(&self, reg: Target) -> String {
        match reg {
            Target::A => return utils::as_bit_string(self.a),
            Target::B => return utils::as_bit_string(self.b),
            Target::C => return utils::as_bit_string(self.c),
            Target::D => return utils::as_bit_string(self.d),
            Target::E => return utils::as_bit_string(self.e),
            Target::F => return utils::as_bit_string(self.f),
            Target::L => return utils::as_bit_string(self.l),
            Target::H => return utils::as_bit_string(self.h),
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn register_as_hex_string(&self, reg: Target) -> String {
        match reg {
            Target::A => return utils::as_hex_string(self.a),
            Target::B => return utils::as_hex_string(self.b),
            Target::C => return utils::as_hex_string(self.c),
            Target::D => return utils::as_hex_string(self.d),
            Target::E => return utils::as_hex_string(self.e),
            Target::F => return utils::as_hex_string(self.f),
            Target::L => return utils::as_hex_string(self.l),
            Target::H => return utils::as_hex_string(self.h),
            _ => {
                panic!("Unimplemented");
            }
        }
    }
}

#[test]
fn test_rotate() {
    let mut reg = Registers::new();
    reg.a = 1;
    reg.rotate_right(Target::A);
    assert!(reg.a == 128);

    reg.rotate_left(Target::A);
    assert!(reg.a == 1);
}

#[test]
fn test_shift() {
    let mut reg = Registers::new();
    reg.a = 1;
    reg.shift_left(Target::A);
    assert!(reg.a == 2);

    reg.shift_right(Target::A);
    assert!(reg.a == 1);
}

#[test]
fn test_bit() {
    let mut reg = Registers::new();
    reg.a = 1;
    assert!(reg.get_bit(Target::A, &0));

    reg.a = 2;
    assert!(!reg.get_bit(Target::A, &0));
    assert!(reg.get_bit(Target::A, &1));
}

#[test]
fn test_set_bit() {
    let mut reg = Registers::new();
    reg.set_bit(Target::A, &0, 1);
    assert!(reg.a == 1);
    reg.set_bit(Target::A, &2, 1);
    assert!(reg.a == 5);
    reg.set_bit(Target::A, &0, 0);
    assert!(reg.a == 4);
}

#[test]
fn test_convert_to_bit_string() {
    let mut reg = Registers::new();

    reg.a = 5;

    assert!(reg.register_as_bit_string(Target::A).as_str().as_bytes() == "0b00000101".as_bytes());
}

#[test]
fn test_convert_to_hex_string() {
    let mut reg = Registers::new();

    reg.a = 128;

    assert!(reg.register_as_hex_string(Target::A).as_str().as_bytes() == "0x80".as_bytes());

    reg.a = 128 + 15;

    assert!(reg.register_as_hex_string(Target::A).as_str().as_bytes() == "0x8F".as_bytes());
}
