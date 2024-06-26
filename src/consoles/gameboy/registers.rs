use super::target::Target;
use crate::utils::conversion;

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
    pub acc: u16,
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
            acc: 0,
        }
    }

    pub fn get_value(&self, src: Target) -> u8 {
        return match src {
            Target::A => self.a,
            Target::B => self.b,
            Target::C => self.b,
            Target::D => self.b,
            Target::E => self.b,
            Target::F => self.b,
            Target::H => self.b,
            Target::L => self.b,
            _ => panic!("Illegal register accessed")
        };
    }
    pub fn get_address(&self, src: Target) -> u16 {
        return match src {
            Target::AF => self.combined_register(Target::AF),
            Target::BC => self.combined_register(Target::BC),
            Target::DE => self.combined_register(Target::DE),
            Target::HL => self.combined_register(Target::HL),
            _ => self.get_value(src) as u16
        };
    }

    pub fn set_register(&mut self, dst: Target, value: u8) {
        match dst {
            Target::A => self.a = value,
            Target::B => self.b = value,
            Target::C => self.b = value,
            Target::D => self.b = value,
            Target::E => self.b = value,
            Target::F => self.b = value,
            Target::H => self.b = value,
            Target::L => self.b = value,
            _ => panic!("Illegal register accessed")
        };
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        return match flag {
            Flag::Zero => self.f >> ZERO_BIT_POS == 1,
            Flag::Carry => self.f & (1 << CARRY_BIT_POS) >= 1,
            Flag::HalfCarry => self.f & (1 << HALF_CARRY_BIT_POS) >= 1,
            Flag::Sub => self.f & (1 << SUB_BIT_POS) >= 1,
            Flag::NotZero => self.f >> ZERO_BIT_POS == 0,
            Flag::NotCarry => self.f & (1 << CARRY_BIT_POS) == 0,
            Flag::NotHalfCarry => self.f & (1 << HALF_CARRY_BIT_POS) == 0,
            Flag::NotSub => self.f & (1 << SUB_BIT_POS) == 0,
        };
    }

    pub fn filter_flag(&self, flag: Flag) -> u8 {
        return match flag {
            Flag::Zero => self.f & (1 << ZERO_BIT_POS) >> ZERO_BIT_POS,
            Flag::Carry => self.f & (1 << CARRY_BIT_POS) >> CARRY_BIT_POS,
            Flag::HalfCarry => self.f & (1 << HALF_CARRY_BIT_POS) >> HALF_CARRY_BIT_POS,
            Flag::Sub => self.f & (1 << SUB_BIT_POS) >> SUB_BIT_POS,
            _ => panic!("Unimplemented");
        };
    }

    pub fn set_flag(&mut self, flag: Flag, v: bool) {
        let mut mask = match flag {
            Flag::Zero => 1 << ZERO_BIT_POS,
            Flag::Carry => 1 << CARRY_BIT_POS,
            Flag::HalfCarry => 1 << HALF_CARRY_BIT_POS,
            Flag::Sub => 1 << SUB_BIT_POS,
            _ => panic!("Invalid flag as Input!");
        }

        if !v {
            mask = !mask;
            self.f = self.f & mask;
        } else {
            self.f = self.f | mask;
        }
    }

    pub fn set_flag_from_u8(&mut self, flag: Flag, bit: u8) {
        self.set_flag(flag, bit == 1);
    }

    pub fn get_bit(&self, reg: Target, bit: &u8) -> bool {
        let mask: u8 = 1 << bit;
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

    pub fn set_bit(&mut self, reg: Target, bit: &u8, v: u8) {
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
            mask = mask ^ (1u8 << bit);
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

    pub fn combined_register(&self, reg: Target) -> u16 {
        match reg {
            Target::HL => ((self.h as u16) << 8) | self.l as u16,
            Target::BC => ((self.b as u16) << 8) | self.c as u16,
            Target::DE => ((self.d as u16) << 8) | self.e as u16,
            Target::AF => ((self.a as u16) << 8) | self.f as u16,
            _ => {
                panic!("Unimplemented {:#?}", reg);
            }
        }
    }

    pub fn set_combined_register(&mut self, reg: Target, v: u16) {
        match reg {
            Target::HL => {
                self.h = ((v & 0b1111111100000000) >> 8) as u8;
                self.l = (v & 0b11111111) as u8;
            }
            Target::BC => {
                self.b = ((v & 0b1111111100000000) >> 8) as u8;
                self.c = (v & 0b11111111) as u8;
            }
            Target::DE => {
                self.d = ((v & 0b1111111100000000) >> 8) as u8;
                self.e = (v & 0b11111111) as u8;
            }
            Target::AF => {
                self.a = ((v & 0b1111111100000000) >> 8) as u8;
                self.f = (v & 0b11111111) as u8;
            }
            _ => {
                panic!("Unimplemented {:#?}", reg);
            }
        }
    }

    pub fn register_as_bit_string(&self, reg: Target) -> String {
        return match reg {
            Target::A => conversion::u8_as_bit_string(self.a),
            Target::B => conversion::u8_as_bit_string(self.b),
            Target::C => conversion::u8_as_bit_string(self.c),
            Target::D => conversion::u8_as_bit_string(self.d),
            Target::E => conversion::u8_as_bit_string(self.e),
            Target::F => conversion::u8_as_bit_string(self.f),
            Target::L => conversion::u8_as_bit_string(self.l),
            Target::H => conversion::u8_as_bit_string(self.h),
            _ => {
                panic!("Unimplemented");
            }
        };
    }

    pub fn register_as_hex_string(&self, reg: Target) -> String {
        return match reg {
            Target::A => conversion::u8_as_hex_string(self.a),
            Target::B => conversion::u8_as_hex_string(self.b),
            Target::C => conversion::u8_as_hex_string(self.c),
            Target::D => conversion::u8_as_hex_string(self.d),
            Target::E => conversion::u8_as_hex_string(self.e),
            Target::F => conversion::u8_as_hex_string(self.f),
            Target::L => conversion::u8_as_hex_string(self.l),
            Target::H => conversion::u8_as_hex_string(self.h),
            _ => {
                panic!("Unimplemented");
            }
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.f = 0;
        self.h = 0;
        self.l = 0;
    }

    pub fn is_16bit_target(&self, reg: Target) -> bool {
        return match reg {
            Target::SP | Target::SP_R8 | Target::D16 => true,
            _ => false,
        }
    }
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
