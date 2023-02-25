use crate::registers::Flag;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    NOP,
    LD(Target, Target),
    LDA,
    ADD(Target),
    ADD16(Target, Target),
    ADC(Target),
    SUB(Target),
    SBC(Target),
    CP(Target), // like sub but result not stored back into a
    AND(Target),
    OR(Target),
    XOR(Target),
    INC(Target),
    DEC(Target),
    CCF,          //Toggle value of carry flag
    SCF,          // set carry flag to true
    RRCA,         // roatet right a reg no through carry flag
    RRLA,         // totate left a rg not through carry flag
    CPL(Target),  // toggle every bit of a reg
    SRL(Target),  // right shift of specific regiser
    RR(Target),   //rotate right specific register through carry flag
    RL(Target),   //rotate left specific register through carry flag
    RRC(Target),  // rotate right specific register not through carry flag
    RLC(Target),  // rotate left specific register not through carry flag
    SRA(Target),  // shift right by 1
    SLA(Target),  // shift left by 1
    SWAP(Target), //swap upper and lower nibble
    BIT(u8, Target),
    RES(u8, Target),
    SET(u8, Target),

    HALT,

    JUMP(Flag),
    JumpUnconditional,
    CALL(Flag),

    PREFIX,
    EnableInterrupt,
    DisableInterrupt,
    EndOfProgram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    A,
    B,
    C,
    D,
    E,
    F,
    L,
    H,
    HL,
    HLP,
    HLM,
    BC,
    DE,
    R8,
    R16,
    D8,
    D16,
    A8,
    A16,
    SP,
    SpR8,
}

#[derive(Debug)]
pub struct Instruction {
    pub opcode: OpCode,
    pub length: u8,
    pub cycles: u16,
    pub optional_cycles: u16,
    pub flags: Vec<u8>,
}

impl Clone for Instruction {
    fn clone(&self) -> Self {
        Instruction {
            opcode: self.opcode,
            length: self.length,
            cycles: self.cycles,
            optional_cycles: self.optional_cycles,
            flags: self.flags.to_owned(),
        }
    }
}

impl Instruction {
    pub fn new(
        opcode: OpCode,
        length: u8,
        cycles: u16,
        optional_cycles: u16,
        flags: Vec<u8>,
    ) -> Instruction {
        Instruction {
            opcode,
            length,
            cycles,
            optional_cycles,
            flags: flags.to_owned(),
        }
    }
}

static AFFECTED: u8 = 2;
static NOT_AFFECTED: u8 = 3;

lazy_static! {
    pub static ref INSTRUCTIONS: HashMap<u8, Instruction> = {
        let mut m = HashMap::new();

        m.insert(0x00 as  u8, Instruction::new(OpCode::NOP, 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        // LD 16 bit
        m.insert(0x01 as u8, Instruction::new(OpCode::LD(Target::BC, Target::D16), 3, 12, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x11 as u8, Instruction::new(OpCode::LD(Target::DE, Target::D16), 3, 12, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x21 as u8, Instruction::new(OpCode::LD(Target::HL, Target::D16), 3, 12, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x31 as u8, Instruction::new(OpCode::LD(Target::SP, Target::D16), 3, 12, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x08 as u8, Instruction::new(OpCode::LD(Target::A16, Target::SP), 3, 12, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));


        // LD 8bit
        m.insert(0x02 as  u8, Instruction::new(OpCode::LD(Target::BC, Target::A), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x06 as  u8, Instruction::new(OpCode::LD(Target::B, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x0A as  u8, Instruction::new(OpCode::LD(Target::A, Target::BC), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x0E as  u8, Instruction::new(OpCode::LD(Target::C, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x12 as  u8, Instruction::new(OpCode::LD(Target::DE, Target::A), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x16 as  u8, Instruction::new(OpCode::LD(Target::D, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x1A as  u8, Instruction::new(OpCode::LD(Target::A, Target::DE), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x1E as  u8, Instruction::new(OpCode::LD(Target::E, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x22 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::A), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED])); // +
        m.insert(0x26 as  u8, Instruction::new(OpCode::LD(Target::H, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x2A as  u8, Instruction::new(OpCode::LD(Target::A, Target::HL), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED])); // +
        m.insert(0x2E as  u8, Instruction::new(OpCode::LD(Target::L, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x32 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::A), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED])); // -
        m.insert(0x36 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x3A as  u8, Instruction::new(OpCode::LD(Target::A, Target::HL), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED])); // -
        m.insert(0x3E as  u8, Instruction::new(OpCode::LD(Target::A, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x40 as  u8, Instruction::new(OpCode::LD(Target::B, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x41 as  u8, Instruction::new(OpCode::LD(Target::B, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x42 as  u8, Instruction::new(OpCode::LD(Target::B, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x43 as  u8, Instruction::new(OpCode::LD(Target::B, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x44 as  u8, Instruction::new(OpCode::LD(Target::B, Target::L), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x45 as  u8, Instruction::new(OpCode::LD(Target::B, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x46 as  u8, Instruction::new(OpCode::LD(Target::B, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x47 as  u8, Instruction::new(OpCode::LD(Target::B, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x48 as  u8, Instruction::new(OpCode::LD(Target::C, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x49 as  u8, Instruction::new(OpCode::LD(Target::C, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x4A as  u8, Instruction::new(OpCode::LD(Target::C, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x4B as  u8, Instruction::new(OpCode::LD(Target::C, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x4C as  u8, Instruction::new(OpCode::LD(Target::C, Target::L), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x4D as  u8, Instruction::new(OpCode::LD(Target::C, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x4E as  u8, Instruction::new(OpCode::LD(Target::C, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x4F as  u8, Instruction::new(OpCode::LD(Target::C, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x50 as  u8, Instruction::new(OpCode::LD(Target::D, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x51 as  u8, Instruction::new(OpCode::LD(Target::D, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x52 as  u8, Instruction::new(OpCode::LD(Target::D, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x53 as  u8, Instruction::new(OpCode::LD(Target::D, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x54 as  u8, Instruction::new(OpCode::LD(Target::D, Target::L), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x55 as  u8, Instruction::new(OpCode::LD(Target::D, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x56 as  u8, Instruction::new(OpCode::LD(Target::D, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x57 as  u8, Instruction::new(OpCode::LD(Target::D, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x58 as  u8, Instruction::new(OpCode::LD(Target::E, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x59 as  u8, Instruction::new(OpCode::LD(Target::E, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x5A as  u8, Instruction::new(OpCode::LD(Target::E, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x5B as  u8, Instruction::new(OpCode::LD(Target::E, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x5C as  u8, Instruction::new(OpCode::LD(Target::E, Target::L), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x5D as  u8, Instruction::new(OpCode::LD(Target::E, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x5E as  u8, Instruction::new(OpCode::LD(Target::E, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x5F as  u8, Instruction::new(OpCode::LD(Target::E, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x60 as  u8, Instruction::new(OpCode::LD(Target::L, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x61 as  u8, Instruction::new(OpCode::LD(Target::L, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x62 as  u8, Instruction::new(OpCode::LD(Target::L, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x63 as  u8, Instruction::new(OpCode::LD(Target::L, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x64 as  u8, Instruction::new(OpCode::LD(Target::L, Target::L), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x65 as  u8, Instruction::new(OpCode::LD(Target::L, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x66 as  u8, Instruction::new(OpCode::LD(Target::L, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x67 as  u8, Instruction::new(OpCode::LD(Target::L, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x68 as  u8, Instruction::new(OpCode::LD(Target::H, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x69 as  u8, Instruction::new(OpCode::LD(Target::H, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x6A as  u8, Instruction::new(OpCode::LD(Target::H, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x6B as  u8, Instruction::new(OpCode::LD(Target::H, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x6C as  u8, Instruction::new(OpCode::LD(Target::H, Target::L), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x6D as  u8, Instruction::new(OpCode::LD(Target::H, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x6E as  u8, Instruction::new(OpCode::LD(Target::H, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x6F as  u8, Instruction::new(OpCode::LD(Target::H, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x70 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x71 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x72 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x73 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x74 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::L), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x75 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x76 as  u8, Instruction::new(OpCode::HALT, 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x77 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x78 as  u8, Instruction::new(OpCode::LD(Target::A, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x79 as  u8, Instruction::new(OpCode::LD(Target::A, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x7A as  u8, Instruction::new(OpCode::LD(Target::A, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x7B as  u8, Instruction::new(OpCode::LD(Target::A, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x7C as  u8, Instruction::new(OpCode::LD(Target::A, Target::L), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x7D as  u8, Instruction::new(OpCode::LD(Target::A, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x7E as  u8, Instruction::new(OpCode::LD(Target::A, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x7F as  u8, Instruction::new(OpCode::LD(Target::A, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        // LDH and so
        m.insert(0xE0 as  u8, Instruction::new(OpCode::LD(Target::A8, Target::A), 2, 12, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF0 as  u8, Instruction::new(OpCode::LD(Target::A, Target::A8), 2, 12, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0xE2 as  u8, Instruction::new(OpCode::LD(Target::C, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF2 as  u8, Instruction::new(OpCode::LD(Target::A, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0xEA as  u8, Instruction::new(OpCode::LD(Target::A16, Target::A), 3, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xFA as  u8, Instruction::new(OpCode::LD(Target::A, Target::A16), 3, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0xF8 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::SpR8), 2, 12, 0, vec![0, 0, AFFECTED , AFFECTED]));
        m.insert(0xF9 as  u8, Instruction::new(OpCode::LD(Target::SP, Target::HL), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));



        // ADD
        m.insert(0x80 as u8,Instruction::new(OpCode::ADD(Target::B), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x81 as u8,Instruction::new(OpCode::ADD(Target::C), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x82 as u8,Instruction::new(OpCode::ADD(Target::D), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x83 as u8,Instruction::new(OpCode::ADD(Target::E), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x84 as u8,Instruction::new(OpCode::ADD(Target::H), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x85 as u8,Instruction::new(OpCode::ADD(Target::L), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x86 as u8,Instruction::new(OpCode::ADD(Target::HL), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x87 as u8,Instruction::new(OpCode::ADD(Target::A), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));

        m.insert(0x88 as u8,Instruction::new(OpCode::ADC(Target::B), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x89 as u8,Instruction::new(OpCode::ADC(Target::C), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8A as u8,Instruction::new(OpCode::ADC(Target::D), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8B as u8,Instruction::new(OpCode::ADC(Target::E), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8C as u8,Instruction::new(OpCode::ADC(Target::L), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8D as u8,Instruction::new(OpCode::ADC(Target::H), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8E as u8,Instruction::new(OpCode::ADC(Target::HL), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8F as u8,Instruction::new(OpCode::ADC(Target::A), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));



        m.insert(0xC6 as u8,Instruction::new(OpCode::ADD(Target::D8), 2, 8, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));


        // add 16 bit
        m.insert(0x09 as u8,Instruction::new(OpCode::ADD16(Target::HL, Target::BC), 1, 8, 0, vec![NOT_AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x19 as u8,Instruction::new(OpCode::ADD16(Target::HL, Target::DE), 1, 8, 0, vec![NOT_AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x29 as u8,Instruction::new(OpCode::ADD16(Target::HL, Target::HL), 1, 8, 0, vec![NOT_AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x39 as u8,Instruction::new(OpCode::ADD16(Target::HL, Target::SP), 1, 8, 0, vec![NOT_AFFECTED, 0, AFFECTED, AFFECTED]));

        m.insert(0xE8 as u8,Instruction::new(OpCode::ADD16(Target::SP, Target::R8), 2, 16, 0, vec![0, 0, AFFECTED, AFFECTED]));


        // Sub
        m.insert(0x90 as u8, Instruction::new(OpCode::SUB(Target::B), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x91 as u8, Instruction::new(OpCode::SUB(Target::C), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x92 as u8, Instruction::new(OpCode::SUB(Target::D), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x93 as u8, Instruction::new(OpCode::SUB(Target::E), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x94 as u8, Instruction::new(OpCode::SUB(Target::L), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x95 as u8, Instruction::new(OpCode::SUB(Target::H), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x96 as u8, Instruction::new(OpCode::SUB(Target::HL), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x97 as u8, Instruction::new(OpCode::SUB(Target::A), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));

        m.insert(0xD6 as u8, Instruction::new(OpCode::SUB(Target::D8), 2, 8, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));



        m.insert(0x98 as u8, Instruction::new(OpCode::SBC(Target::B), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x99 as u8, Instruction::new(OpCode::SBC(Target::C), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9A as u8, Instruction::new(OpCode::SBC(Target::D), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9B as u8, Instruction::new(OpCode::SBC(Target::E), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9C as u8, Instruction::new(OpCode::SBC(Target::L), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9D as u8, Instruction::new(OpCode::SBC(Target::H), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9E as u8, Instruction::new(OpCode::SBC(Target::HL), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9F as u8, Instruction::new(OpCode::SBC(Target::A), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));




        // AND
        m.insert(0xA0 as u8, Instruction::new(OpCode::AND(Target::B), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA1 as u8, Instruction::new(OpCode::AND(Target::C), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA2 as u8, Instruction::new(OpCode::AND(Target::D), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA3 as u8, Instruction::new(OpCode::AND(Target::E), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA4 as u8, Instruction::new(OpCode::AND(Target::L), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA5 as u8, Instruction::new(OpCode::AND(Target::H), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA6 as u8, Instruction::new(OpCode::AND(Target::HL), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA7 as u8, Instruction::new(OpCode::AND(Target::A), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));

        m.insert(0xE6 as u8, Instruction::new(OpCode::AND(Target::D8), 2, 8, 0, vec![AFFECTED, 0, 1, 0]));


        // XOR
        m.insert(0xA8 as u8, Instruction::new(OpCode::XOR(Target::B), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xA9 as u8, Instruction::new(OpCode::XOR(Target::C), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAA as u8, Instruction::new(OpCode::XOR(Target::D), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAB as u8, Instruction::new(OpCode::XOR(Target::E), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAC as u8, Instruction::new(OpCode::XOR(Target::L), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAD as u8, Instruction::new(OpCode::XOR(Target::H), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAE as u8, Instruction::new(OpCode::XOR(Target::HL), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAF as u8, Instruction::new(OpCode::XOR(Target::A), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));

        // OR
        m.insert(0xB0 as u8, Instruction::new(OpCode::OR(Target::B), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB1 as u8, Instruction::new(OpCode::OR(Target::C), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB2 as u8, Instruction::new(OpCode::OR(Target::D), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB3 as u8, Instruction::new(OpCode::OR(Target::E), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB4 as u8, Instruction::new(OpCode::OR(Target::L), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB5 as u8, Instruction::new(OpCode::OR(Target::H), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB6 as u8, Instruction::new(OpCode::OR(Target::HL), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB7 as u8, Instruction::new(OpCode::OR(Target::A), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));

        m.insert(0xF6 as u8, Instruction::new(OpCode::OR(Target::D8), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));


        // CP
        m.insert(0xB8 as u8, Instruction::new(OpCode::CP(Target::B), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xB9 as u8, Instruction::new(OpCode::CP(Target::C), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBA as u8, Instruction::new(OpCode::CP(Target::D), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBB as u8, Instruction::new(OpCode::CP(Target::E), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBC as u8, Instruction::new(OpCode::CP(Target::L), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBD as u8, Instruction::new(OpCode::CP(Target::H), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBE as u8, Instruction::new(OpCode::CP(Target::HL), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBF as u8, Instruction::new(OpCode::CP(Target::A), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));

        // INC
        m.insert(0x03 as u8, Instruction::new(OpCode::INC(Target::BC), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x04 as u8, Instruction::new(OpCode::INC(Target::B), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));
        m.insert(0x0C as u8, Instruction::new(OpCode::INC(Target::C), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));

        m.insert(0x13 as u8, Instruction::new(OpCode::INC(Target::DE), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x14 as u8, Instruction::new(OpCode::INC(Target::D), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));
        m.insert(0x1C as u8, Instruction::new(OpCode::INC(Target::E), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));

        m.insert(0x23 as u8, Instruction::new(OpCode::INC(Target::HL), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x24 as u8, Instruction::new(OpCode::INC(Target::H), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));
        m.insert(0x2C as u8, Instruction::new(OpCode::INC(Target::L), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));


        m.insert(0x33 as u8, Instruction::new(OpCode::INC(Target::SP), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x34 as u8, Instruction::new(OpCode::INC(Target::HL), 1, 12, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));
        m.insert(0x3C as u8, Instruction::new(OpCode::INC(Target::A), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));

        // DEC
        m.insert(0x04 as u8, Instruction::new(OpCode::DEC(Target::B), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));
        m.insert(0x0D as u8, Instruction::new(OpCode::DEC(Target::C), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));


        m.insert(0x14 as u8, Instruction::new(OpCode::DEC(Target::D), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));
        m.insert(0x1D as u8, Instruction::new(OpCode::DEC(Target::E), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));

        m.insert(0x24 as u8, Instruction::new(OpCode::DEC(Target::H), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));
        m.insert(0x2D as u8, Instruction::new(OpCode::DEC(Target::L), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));

        m.insert(0x34 as u8, Instruction::new(OpCode::DEC(Target::HL), 1, 12, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));
        m.insert(0x3D as u8, Instruction::new(OpCode::DEC(Target::A), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));


        m.insert(0xC2 as u8, Instruction::new(OpCode::JUMP(Flag::NotZero), 3, 16, 12, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC3 as u8, Instruction::new(OpCode::JumpUnconditional, 3, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        // Prefix
        m.insert(0xCB as u8, Instruction::new(OpCode::PREFIX, 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        // Enable / disable interrupts
        m.insert(0xF3 as u8, Instruction::new(OpCode::DisableInterrupt, 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xFB as u8, Instruction::new(OpCode::EnableInterrupt, 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));


        // End of program instruction, only used for debugging
        m.insert(0xFD as u8, Instruction::new(OpCode::EndOfProgram, 0, 0, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));



        m
    };
    static ref INSTRUCTION_COUNT: usize = INSTRUCTIONS.len();
}

lazy_static! {
    static ref PREFIXED_INSTRUCTIONS: HashMap<u8, Instruction> = {
        let mut m = HashMap::new();


        // RLC
        m.insert(
            0x00 as u8,
            Instruction::new(
                OpCode::RLC(Target::B),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x01 as u8,
            Instruction::new(
                OpCode::RLC(Target::C),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x02 as u8,
            Instruction::new(
                OpCode::RLC(Target::D),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x03 as u8,
            Instruction::new(
                OpCode::RLC(Target::E),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x04 as u8,
            Instruction::new(
                OpCode::RLC(Target::H),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x05 as u8,
            Instruction::new(
                OpCode::RLC(Target::L),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x06 as u8,
            Instruction::new(
                OpCode::RLC(Target::HL),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x07 as u8,
            Instruction::new(
                OpCode::RLC(Target::A),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );

        //RRC
        m.insert(
            0x08 as u8,
            Instruction::new(
                OpCode::RRC(Target::B),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x09 as u8,
            Instruction::new(
                OpCode::RRC(Target::C),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x0A as u8,
            Instruction::new(
                OpCode::RRC(Target::D),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x0B as u8,
            Instruction::new(
                OpCode::RRC(Target::E),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x0C as u8,
            Instruction::new(
                OpCode::RRC(Target::H),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x0D as u8,
            Instruction::new(
                OpCode::RRC(Target::L),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x0E as u8,
            Instruction::new(
                OpCode::RRC(Target::HL),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x0F as u8,
            Instruction::new(
                OpCode::RRC(Target::A),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );


        // RL
        m.insert(
            0x10 as u8,
            Instruction::new(
                OpCode::RL(Target::B),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x11 as u8,
            Instruction::new(
                OpCode::RL(Target::C),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x12 as u8,
            Instruction::new(
                OpCode::RL(Target::D),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x13 as u8,
            Instruction::new(
                OpCode::RL(Target::E),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x14 as u8,
            Instruction::new(
                OpCode::RL(Target::H),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x15 as u8,
            Instruction::new(
                OpCode::RL(Target::L),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x16 as u8,
            Instruction::new(
                OpCode::RL(Target::HL),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x17 as u8,
            Instruction::new(
                OpCode::RL(Target::A),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );


        //RR
        m.insert(
            0x18 as u8,
            Instruction::new(
                OpCode::RR(Target::B),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x19 as u8,
            Instruction::new(
                OpCode::RR(Target::C),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x1A as u8,
            Instruction::new(
                OpCode::RR(Target::D),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x1B as u8,
            Instruction::new(
                OpCode::RR(Target::E),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x1C as u8,
            Instruction::new(
                OpCode::RR(Target::H),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x1D as u8,
            Instruction::new(
                OpCode::RR(Target::L),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x1E as u8,
            Instruction::new(
                OpCode::RR(Target::HL),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x1F as u8,
            Instruction::new(
                OpCode::RR(Target::A),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );

        //SLA
        m.insert(
            0x20 as u8,
            Instruction::new(
                OpCode::SLA(Target::B),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x21 as u8,
            Instruction::new(
                OpCode::SLA(Target::C),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x22 as u8,
            Instruction::new(
                OpCode::SLA(Target::D),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x23 as u8,
            Instruction::new(
                OpCode::SLA(Target::E),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x24 as u8,
            Instruction::new(
                OpCode::SLA(Target::H),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x25 as u8,
            Instruction::new(
                OpCode::SLA(Target::L),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x26 as u8,
            Instruction::new(
                OpCode::SLA(Target::HL),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );
        m.insert(
            0x27 as u8,
            Instruction::new(
                OpCode::SLA(Target::A),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, AFFECTED],
            ),
        );

        //SRA
        m.insert(
            0x28 as u8,
            Instruction::new(
                OpCode::SRA(Target::B),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, 0],
            ),
        );
        m.insert(
            0x29 as u8,
            Instruction::new(
                OpCode::SRA(Target::C),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, 0],
            ),
        );
        m.insert(
            0x2A as u8,
            Instruction::new(
                OpCode::SRA(Target::D),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, 0],
            ),
        );
        m.insert(
            0x2B as u8,
            Instruction::new(
                OpCode::SRA(Target::E),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, 0],
            ),
        );
        m.insert(
            0x2C as u8,
            Instruction::new(
                OpCode::SRA(Target::H),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, 0],
            ),
        );
        m.insert(
            0x2D as u8,
            Instruction::new(
                OpCode::SRA(Target::L),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, 0],
            ),
        );
        m.insert(
            0x2E as u8,
            Instruction::new(
                OpCode::SRA(Target::HL),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, 0],
            ),
        );
        m.insert(
            0x2F as u8,
            Instruction::new(
                OpCode::SRA(Target::A),
                2,
                8,
                0,
                vec![AFFECTED, 0, 0, 0],
            ),
        );

        // Swap
        m.insert(0x30, Instruction::new(OpCode::SWAP(Target::B), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0x31, Instruction::new(OpCode::SWAP(Target::C), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0x32, Instruction::new(OpCode::SWAP(Target::D), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0x33, Instruction::new(OpCode::SWAP(Target::E), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0x34, Instruction::new(OpCode::SWAP(Target::H), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0x35, Instruction::new(OpCode::SWAP(Target::L), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0x36, Instruction::new(OpCode::SWAP(Target::HL), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0x37, Instruction::new(OpCode::SWAP(Target::A), 2, 8, 0, vec![AFFECTED, 0, 0, 0]));

        //SRL
        m.insert(0x38, Instruction::new(OpCode::SRL(Target::B), 2, 8, 0, vec![AFFECTED, 0, 0, AFFECTED]));
        m.insert(0x39, Instruction::new(OpCode::SRL(Target::C), 2, 8, 0, vec![AFFECTED, 0, 0, AFFECTED]));
        m.insert(0x3A, Instruction::new(OpCode::SRL(Target::D), 2, 8, 0, vec![AFFECTED, 0, 0, AFFECTED]));
        m.insert(0x3B, Instruction::new(OpCode::SRL(Target::E), 2, 8, 0, vec![AFFECTED, 0, 0, AFFECTED]));
        m.insert(0x3C, Instruction::new(OpCode::SRL(Target::H), 2, 8, 0, vec![AFFECTED, 0, 0, AFFECTED]));
        m.insert(0x3D, Instruction::new(OpCode::SRL(Target::L), 2, 8, 0, vec![AFFECTED, 0, 0, AFFECTED]));
        m.insert(0x3E, Instruction::new(OpCode::SRL(Target::HL), 2, 8, 0, vec![AFFECTED, 0, 0, AFFECTED]));
        m.insert(0x3F, Instruction::new(OpCode::SRL(Target::A), 2, 8, 0, vec![AFFECTED, 0, 0, AFFECTED]));

        // BIT
        m.insert(0x40, Instruction::new(OpCode::BIT(0, Target::B), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x41, Instruction::new(OpCode::BIT(0, Target::C), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x42, Instruction::new(OpCode::BIT(0, Target::D), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x43, Instruction::new(OpCode::BIT(0, Target::E), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x44, Instruction::new(OpCode::BIT(0, Target::H), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x45, Instruction::new(OpCode::BIT(0, Target::L), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x46, Instruction::new(OpCode::BIT(0, Target::HL), 2, 16, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x47, Instruction::new(OpCode::BIT(0, Target::A), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x48, Instruction::new(OpCode::BIT(1, Target::B), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x49, Instruction::new(OpCode::BIT(1, Target::C), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x4A, Instruction::new(OpCode::BIT(1, Target::D), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x4B, Instruction::new(OpCode::BIT(1, Target::E), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x4C, Instruction::new(OpCode::BIT(1, Target::H), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x4D, Instruction::new(OpCode::BIT(1, Target::L), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x4E, Instruction::new(OpCode::BIT(1, Target::HL), 2, 16, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x4F, Instruction::new(OpCode::BIT(1, Target::A), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));


        m.insert(0x50, Instruction::new(OpCode::BIT(2, Target::B), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x51, Instruction::new(OpCode::BIT(2, Target::C), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x52, Instruction::new(OpCode::BIT(2, Target::D), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x53, Instruction::new(OpCode::BIT(2, Target::E), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x54, Instruction::new(OpCode::BIT(2, Target::H), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x55, Instruction::new(OpCode::BIT(2, Target::L), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x56, Instruction::new(OpCode::BIT(2, Target::HL), 2, 16, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x57, Instruction::new(OpCode::BIT(2, Target::A), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x58, Instruction::new(OpCode::BIT(3, Target::B), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x59, Instruction::new(OpCode::BIT(3, Target::C), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x5A, Instruction::new(OpCode::BIT(3, Target::D), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x5B, Instruction::new(OpCode::BIT(3, Target::E), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x5C, Instruction::new(OpCode::BIT(3, Target::H), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x5D, Instruction::new(OpCode::BIT(3, Target::L), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x5E, Instruction::new(OpCode::BIT(3, Target::HL), 2, 16, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x5F, Instruction::new(OpCode::BIT(3, Target::A), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));

        m.insert(0x60, Instruction::new(OpCode::BIT(4, Target::B), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x61, Instruction::new(OpCode::BIT(4, Target::C), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x62, Instruction::new(OpCode::BIT(4, Target::D), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x63, Instruction::new(OpCode::BIT(4, Target::E), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x64, Instruction::new(OpCode::BIT(4, Target::H), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x65, Instruction::new(OpCode::BIT(4, Target::L), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x66, Instruction::new(OpCode::BIT(4, Target::HL), 2, 16, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x67, Instruction::new(OpCode::BIT(4, Target::A), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x68, Instruction::new(OpCode::BIT(5, Target::B), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x69, Instruction::new(OpCode::BIT(5, Target::C), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x6A, Instruction::new(OpCode::BIT(5, Target::D), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x6B, Instruction::new(OpCode::BIT(5, Target::E), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x6C, Instruction::new(OpCode::BIT(5, Target::H), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x6D, Instruction::new(OpCode::BIT(5, Target::L), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x6E, Instruction::new(OpCode::BIT(5, Target::HL), 2, 16, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x6F, Instruction::new(OpCode::BIT(5, Target::A), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));


        m.insert(0x70, Instruction::new(OpCode::BIT(6, Target::B), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x71, Instruction::new(OpCode::BIT(6, Target::C), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x72, Instruction::new(OpCode::BIT(6, Target::D), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x73, Instruction::new(OpCode::BIT(6, Target::E), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x74, Instruction::new(OpCode::BIT(6, Target::H), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x75, Instruction::new(OpCode::BIT(6, Target::L), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x76, Instruction::new(OpCode::BIT(6, Target::HL), 2, 16, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x77, Instruction::new(OpCode::BIT(6, Target::A), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x78, Instruction::new(OpCode::BIT(7, Target::B), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x79, Instruction::new(OpCode::BIT(7, Target::C), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x7A, Instruction::new(OpCode::BIT(7, Target::D), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x7B, Instruction::new(OpCode::BIT(7, Target::E), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x7C, Instruction::new(OpCode::BIT(7, Target::H), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x7D, Instruction::new(OpCode::BIT(7, Target::L), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x7E, Instruction::new(OpCode::BIT(7, Target::HL), 2, 16, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));
        m.insert(0x7F, Instruction::new(OpCode::BIT(7, Target::A), 2, 8, 0, vec![AFFECTED, 0, 1, NOT_AFFECTED]));



        // RES
        m.insert(0x80, Instruction::new(OpCode::RES(0, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x81, Instruction::new(OpCode::RES(0, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x82, Instruction::new(OpCode::RES(0, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x83, Instruction::new(OpCode::RES(0, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x84, Instruction::new(OpCode::RES(0, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x85, Instruction::new(OpCode::RES(0, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x86, Instruction::new(OpCode::RES(0, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x87, Instruction::new(OpCode::RES(0, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x88, Instruction::new(OpCode::RES(1, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x89, Instruction::new(OpCode::RES(1, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x8A, Instruction::new(OpCode::RES(1, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x8B, Instruction::new(OpCode::RES(1, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x8C, Instruction::new(OpCode::RES(1, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x8D, Instruction::new(OpCode::RES(1, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x8E, Instruction::new(OpCode::RES(1, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x8F, Instruction::new(OpCode::RES(1, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0x90, Instruction::new(OpCode::RES(2, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x91, Instruction::new(OpCode::RES(2, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x92, Instruction::new(OpCode::RES(2, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x93, Instruction::new(OpCode::RES(2, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x94, Instruction::new(OpCode::RES(2, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x95, Instruction::new(OpCode::RES(2, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x96, Instruction::new(OpCode::RES(2, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x97, Instruction::new(OpCode::RES(2, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x98, Instruction::new(OpCode::RES(3, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x99, Instruction::new(OpCode::RES(3, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x9A, Instruction::new(OpCode::RES(3, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x9B, Instruction::new(OpCode::RES(3, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x9C, Instruction::new(OpCode::RES(3, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x9D, Instruction::new(OpCode::RES(3, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x9E, Instruction::new(OpCode::RES(3, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x9F, Instruction::new(OpCode::RES(3, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0xA0, Instruction::new(OpCode::RES(4, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA1, Instruction::new(OpCode::RES(4, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA2, Instruction::new(OpCode::RES(4, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA3, Instruction::new(OpCode::RES(4, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA4, Instruction::new(OpCode::RES(4, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA5, Instruction::new(OpCode::RES(4, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA6, Instruction::new(OpCode::RES(4, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA7, Instruction::new(OpCode::RES(4, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA8, Instruction::new(OpCode::RES(5, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xA9, Instruction::new(OpCode::RES(5, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xAA, Instruction::new(OpCode::RES(5, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xAB, Instruction::new(OpCode::RES(5, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xAC, Instruction::new(OpCode::RES(5, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xAD, Instruction::new(OpCode::RES(5, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xAE, Instruction::new(OpCode::RES(5, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xAF, Instruction::new(OpCode::RES(5, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0xB0, Instruction::new(OpCode::RES(6, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB1, Instruction::new(OpCode::RES(6, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB2, Instruction::new(OpCode::RES(6, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB3, Instruction::new(OpCode::RES(6, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB4, Instruction::new(OpCode::RES(6, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB5, Instruction::new(OpCode::RES(6, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB6, Instruction::new(OpCode::RES(6, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB7, Instruction::new(OpCode::RES(6, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB8, Instruction::new(OpCode::RES(7, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xB9, Instruction::new(OpCode::RES(7, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xBA, Instruction::new(OpCode::RES(7, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xBB, Instruction::new(OpCode::RES(7, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xBC, Instruction::new(OpCode::RES(7, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xBD, Instruction::new(OpCode::RES(7, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xBE, Instruction::new(OpCode::RES(7, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xBF, Instruction::new(OpCode::RES(7, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));


        // SET
        m.insert(0xC0, Instruction::new(OpCode::SET(0, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC1, Instruction::new(OpCode::SET(0, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC2, Instruction::new(OpCode::SET(0, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC3, Instruction::new(OpCode::SET(0, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC4, Instruction::new(OpCode::SET(0, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC5, Instruction::new(OpCode::SET(0, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC6, Instruction::new(OpCode::SET(0, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC7, Instruction::new(OpCode::SET(0, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC8, Instruction::new(OpCode::SET(1, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC9, Instruction::new(OpCode::SET(1, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xCA, Instruction::new(OpCode::SET(1, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xCB, Instruction::new(OpCode::SET(1, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xCC, Instruction::new(OpCode::SET(1, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xCD, Instruction::new(OpCode::SET(1, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xCE, Instruction::new(OpCode::SET(1, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xCF, Instruction::new(OpCode::SET(1, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0xD0, Instruction::new(OpCode::SET(2, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD1, Instruction::new(OpCode::SET(2, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD2, Instruction::new(OpCode::SET(2, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD3, Instruction::new(OpCode::SET(2, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD4, Instruction::new(OpCode::SET(2, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD5, Instruction::new(OpCode::SET(2, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD6, Instruction::new(OpCode::SET(2, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD7, Instruction::new(OpCode::SET(2, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD8, Instruction::new(OpCode::SET(3, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xD9, Instruction::new(OpCode::SET(3, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xDA, Instruction::new(OpCode::SET(3, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xDB, Instruction::new(OpCode::SET(3, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xDC, Instruction::new(OpCode::SET(3, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xDD, Instruction::new(OpCode::SET(3, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xDE, Instruction::new(OpCode::SET(3, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xDF, Instruction::new(OpCode::SET(3, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0xE0, Instruction::new(OpCode::SET(4, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE1, Instruction::new(OpCode::SET(4, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE2, Instruction::new(OpCode::SET(4, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE3, Instruction::new(OpCode::SET(4, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE4, Instruction::new(OpCode::SET(4, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE5, Instruction::new(OpCode::SET(4, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE6, Instruction::new(OpCode::SET(4, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE7, Instruction::new(OpCode::SET(4, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE8, Instruction::new(OpCode::SET(5, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xE9, Instruction::new(OpCode::SET(5, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xEA, Instruction::new(OpCode::SET(5, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xEB, Instruction::new(OpCode::SET(5, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xEC, Instruction::new(OpCode::SET(5, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xED, Instruction::new(OpCode::SET(5, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xEE, Instruction::new(OpCode::SET(5, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xEF, Instruction::new(OpCode::SET(5, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));

        m.insert(0xF0, Instruction::new(OpCode::SET(6, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF1, Instruction::new(OpCode::SET(6, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF2, Instruction::new(OpCode::SET(6, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF3, Instruction::new(OpCode::SET(6, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF4, Instruction::new(OpCode::SET(6, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF5, Instruction::new(OpCode::SET(6, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF6, Instruction::new(OpCode::SET(6, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF7, Instruction::new(OpCode::SET(6, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF8, Instruction::new(OpCode::SET(7, Target::B), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xF9, Instruction::new(OpCode::SET(7, Target::C), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xFA, Instruction::new(OpCode::SET(7, Target::D), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xFB, Instruction::new(OpCode::SET(7, Target::E), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xFC, Instruction::new(OpCode::SET(7, Target::H), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xFD, Instruction::new(OpCode::SET(7, Target::L), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xFE, Instruction::new(OpCode::SET(7, Target::HL), 2, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xFF, Instruction::new(OpCode::SET(7, Target::A), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));


        m
    };
    static ref PREFIXED_INSTRUCTION_COUNT: usize = PREFIXED_INSTRUCTIONS.len();
}

impl Instruction {
    pub fn look_up(byte: u8) -> Option<&'static Instruction> {
        if let Some(instruction) = Self::fetch(byte, false) {
            return Some(instruction);
        } else if let Some(instruction) = Self::fetch(byte, false) {
            return Some(instruction);
        }

        None
    }

    pub fn fetch(byte: u8, prefixed: bool) -> Option<&'static Instruction> {
        if prefixed {
            return Self::from_byte_prefixed(byte);
        }

        Self::from_byte(byte)
    }

    fn from_byte(byte: u8) -> Option<&'static Instruction> {
        let i = INSTRUCTIONS.get(&byte);
        if let Some(instruction) = i {
            return Some(instruction);
        }
        None
    }

    fn from_byte_prefixed(byte: u8) -> Option<&'static Instruction> {
        let i = PREFIXED_INSTRUCTIONS.get(&byte);
        if let Some(instruction) = i {
            return Some(instruction);
        }
        None
    }

    pub fn from_opcode(op: OpCode) -> Option<&'static Instruction> {
        if let Some(instruction) = Self::from_opcode_unprefixed(op) {
            return Some(instruction);
        } else if let Some(instruction) = Self::from_opcode_prefixed(op) {
            return Some(instruction);
        }
        return None;
    }

    fn from_opcode_unprefixed(op: OpCode) -> Option<&'static Instruction> {
        for (_code, instruction) in &*INSTRUCTIONS {
            if instruction.opcode == op {
                return Some(instruction);
            }
        }
        return None;
    }

    fn from_opcode_prefixed(_op: OpCode) -> Option<&'static Instruction> {
        for (_code, instruction) in &*PREFIXED_INSTRUCTIONS {
            if instruction.opcode == _op {
                return Some(instruction);
            }
        }
        return None;
    }

    pub fn byte_from_opcode(op: OpCode) -> Option<u8> {
        if let Some(byte) = Self::byte_from_opcode_unprefixed(op) {
            return Some(byte);
        } else if let Some(byte) = Self::byte_from_opcode_prefixed(op) {
            return Some(byte);
        }

        None
    }

    fn byte_from_opcode_unprefixed(opcode: OpCode) -> Option<u8> {
        for (code, instruction) in &*INSTRUCTIONS {
            if opcode == instruction.opcode {
                return Some(code.to_owned());
            }
        }
        None
    }

    fn byte_from_opcode_prefixed(opcode: OpCode) -> Option<u8> {
        for (code, instruction) in &*PREFIXED_INSTRUCTIONS {
            if opcode == instruction.opcode {
                return Some(code.to_owned());
            }
        }
        None
    }

    pub fn print_instruction_bytes_as_char() {
        let mut data = String::new();

        println!("Instuction bytes");
        for (code, _instruction) in &*INSTRUCTIONS {
            data = data
                + format!(
                    "{:#x}: {} = {:#?}\n",
                    code,
                    code.to_owned() as char,
                    _instruction
                )
                .as_str();
            println!(
                "byte: {} = {} :  {:#?}",
                code,
                code.to_owned() as char,
                _instruction.opcode
            );
        }
        println!("-------------");

        let file = File::create("instructions.txt");
        file.expect("File coudld not be opened")
            .write(data.as_bytes())
            .expect("Failed to write file");
    }

    pub fn test_instruction_completeness() {
        for i in 0..0xFF {
            let instruction = Instruction::from_byte(i);
            match instruction {
                None => {
                    println!("{:#x} is not implemented", i);
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test_from_opcode() {
    let op = Instruction::from_opcode(OpCode::ADD(Target::B));

    assert!(op.is_some());
    eprintln!("{:#?}", op.unwrap());
    assert!(matches!(op.unwrap().opcode, OpCode::ADD(Target::B)));
}

#[test]
fn test_fetch() {
    let instruction =
        Instruction::fetch(Instruction::byte_from_opcode(OpCode::NOP).unwrap(), false).unwrap();
    assert!(instruction.opcode == OpCode::NOP);

    let instruction = Instruction::fetch(
        Instruction::byte_from_opcode(OpCode::SWAP(Target::A)).unwrap(),
        true,
    )
    .unwrap();
    assert!(instruction.opcode == OpCode::SWAP(Target::A));
}
