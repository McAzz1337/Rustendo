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
    ADDHL(Target),
    ADC(Target, Target),
    SUB(Target),
    SBC(Target, Target),
    CP(Target), // like sub but result not stored back into a
    AND(Target),
    OR(Target),
    XOR(Target),
    INC(Target),
    DEC(Target),
    CCF,          //Toggle value of carry flag
    SCF,          // set carry flag to true
    RRA,          // rotate right a reg
    RLA,          // rotate left a reg
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

    JUMP(Flag),
    JUMP_UNCONDICIONAL,
    CALL(Flag),

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
    G,
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
    static ref INSTRUCTIONS: HashMap<u8, Instruction> = {
        let mut m = HashMap::new();

        m.insert(0x00 as  u8, Instruction::new(OpCode::NOP, 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        // LD 8bit
        m.insert(0x02 as  u8, Instruction::new(OpCode::LD(Target::BC, Target::A), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x06 as  u8, Instruction::new(OpCode::LD(Target::B, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x0A as  u8, Instruction::new(OpCode::LD(Target::A, Target::BC), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x0E as  u8, Instruction::new(OpCode::LD(Target::C, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x12 as  u8, Instruction::new(OpCode::LD(Target::DE, Target::A), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x16 as  u8, Instruction::new(OpCode::LD(Target::D, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x1A as  u8, Instruction::new(OpCode::LD(Target::A, Target::DE), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x1E as  u8, Instruction::new(OpCode::LD(Target::E, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x22 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::A), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED])); // +
        m.insert(0x26 as  u8, Instruction::new(OpCode::LD(Target::H, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x2A as  u8, Instruction::new(OpCode::LD(Target::A, Target::HL), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED])); // +
        m.insert(0x2E as  u8, Instruction::new(OpCode::LD(Target::G, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x32 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::A), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED])); // -
        m.insert(0x36 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x3A as  u8, Instruction::new(OpCode::LD(Target::A, Target::HL), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED])); // -
        m.insert(0x3E as  u8, Instruction::new(OpCode::LD(Target::A, Target::D8), 2, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x40 as  u8, Instruction::new(OpCode::LD(Target::B, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x41 as  u8, Instruction::new(OpCode::LD(Target::B, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x42 as  u8, Instruction::new(OpCode::LD(Target::B, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x43 as  u8, Instruction::new(OpCode::LD(Target::B, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x44 as  u8, Instruction::new(OpCode::LD(Target::B, Target::G), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x45 as  u8, Instruction::new(OpCode::LD(Target::B, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x46 as  u8, Instruction::new(OpCode::LD(Target::B, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x47 as  u8, Instruction::new(OpCode::LD(Target::B, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x48 as  u8, Instruction::new(OpCode::LD(Target::C, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x49 as  u8, Instruction::new(OpCode::LD(Target::C, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x4A as  u8, Instruction::new(OpCode::LD(Target::C, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x4B as  u8, Instruction::new(OpCode::LD(Target::C, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x4C as  u8, Instruction::new(OpCode::LD(Target::C, Target::G), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x4D as  u8, Instruction::new(OpCode::LD(Target::C, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x4E as  u8, Instruction::new(OpCode::LD(Target::C, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x4F as  u8, Instruction::new(OpCode::LD(Target::C, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x50 as  u8, Instruction::new(OpCode::LD(Target::D, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x51 as  u8, Instruction::new(OpCode::LD(Target::D, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x52 as  u8, Instruction::new(OpCode::LD(Target::D, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x53 as  u8, Instruction::new(OpCode::LD(Target::D, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x54 as  u8, Instruction::new(OpCode::LD(Target::D, Target::G), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x55 as  u8, Instruction::new(OpCode::LD(Target::D, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x56 as  u8, Instruction::new(OpCode::LD(Target::D, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x57 as  u8, Instruction::new(OpCode::LD(Target::D, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x58 as  u8, Instruction::new(OpCode::LD(Target::E, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x59 as  u8, Instruction::new(OpCode::LD(Target::E, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x5A as  u8, Instruction::new(OpCode::LD(Target::E, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x5B as  u8, Instruction::new(OpCode::LD(Target::E, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x5C as  u8, Instruction::new(OpCode::LD(Target::E, Target::G), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x5D as  u8, Instruction::new(OpCode::LD(Target::E, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x5E as  u8, Instruction::new(OpCode::LD(Target::E, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x5F as  u8, Instruction::new(OpCode::LD(Target::E, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x60 as  u8, Instruction::new(OpCode::LD(Target::G, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x61 as  u8, Instruction::new(OpCode::LD(Target::G, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x62 as  u8, Instruction::new(OpCode::LD(Target::G, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x63 as  u8, Instruction::new(OpCode::LD(Target::G, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x64 as  u8, Instruction::new(OpCode::LD(Target::G, Target::G), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x65 as  u8, Instruction::new(OpCode::LD(Target::G, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x66 as  u8, Instruction::new(OpCode::LD(Target::G, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x67 as  u8, Instruction::new(OpCode::LD(Target::G, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x68 as  u8, Instruction::new(OpCode::LD(Target::H, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x69 as  u8, Instruction::new(OpCode::LD(Target::H, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x6A as  u8, Instruction::new(OpCode::LD(Target::H, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x6B as  u8, Instruction::new(OpCode::LD(Target::H, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x6C as  u8, Instruction::new(OpCode::LD(Target::H, Target::G), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x6D as  u8, Instruction::new(OpCode::LD(Target::H, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x6E as  u8, Instruction::new(OpCode::LD(Target::H, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x6F as  u8, Instruction::new(OpCode::LD(Target::H, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x70 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x71 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x72 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x73 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x74 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::G), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x75 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x76 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x77 as  u8, Instruction::new(OpCode::LD(Target::HL, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));

        m.insert(0x78 as  u8, Instruction::new(OpCode::LD(Target::A, Target::B), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x79 as  u8, Instruction::new(OpCode::LD(Target::A, Target::C), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x7A as  u8, Instruction::new(OpCode::LD(Target::A, Target::D), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x7B as  u8, Instruction::new(OpCode::LD(Target::A, Target::E), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x7C as  u8, Instruction::new(OpCode::LD(Target::A, Target::G), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x7D as  u8, Instruction::new(OpCode::LD(Target::A, Target::H), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x7E as  u8, Instruction::new(OpCode::LD(Target::A, Target::HL), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));
        m.insert(0x7F as  u8, Instruction::new(OpCode::LD(Target::A, Target::A), 1, 4, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED ,NOT_AFFECTED]));


        // ADD
        m.insert(0x80 as u8,Instruction::new(OpCode::ADD(Target::B), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x81 as u8,Instruction::new(OpCode::ADD(Target::C), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x82 as u8,Instruction::new(OpCode::ADD(Target::D), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x83 as u8,Instruction::new(OpCode::ADD(Target::E), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x84 as u8,Instruction::new(OpCode::ADD(Target::H), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x85 as u8,Instruction::new(OpCode::ADD(Target::G), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x86 as u8,Instruction::new(OpCode::ADD(Target::HL), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x87 as u8,Instruction::new(OpCode::ADD(Target::A), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));

        m.insert(0x88 as u8,Instruction::new(OpCode::ADC(Target::A, Target::B), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x89 as u8,Instruction::new(OpCode::ADC(Target::A, Target::C), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8A as u8,Instruction::new(OpCode::ADC(Target::A, Target::D), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8B as u8,Instruction::new(OpCode::ADC(Target::A, Target::E), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8C as u8,Instruction::new(OpCode::ADC(Target::A, Target::G), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8D as u8,Instruction::new(OpCode::ADC(Target::A, Target::H), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8E as u8,Instruction::new(OpCode::ADC(Target::A, Target::HL), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x8F as u8,Instruction::new(OpCode::ADC(Target::A, Target::A), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));

        m.insert(0x89 as u8,Instruction::new(OpCode::ADD(Target::D8), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));

        m.insert(0x89 as u8,Instruction::new(OpCode::ADD(Target::BC), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x89 as u8,Instruction::new(OpCode::ADD(Target::DE), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x89 as u8,Instruction::new(OpCode::ADD(Target::HL), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));
        m.insert(0x89 as u8,Instruction::new(OpCode::ADD(Target::SP), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, AFFECTED]));

        // Sub
        m.insert(0x90 as u8, Instruction::new(OpCode::SUB(Target::B), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x91 as u8, Instruction::new(OpCode::SUB(Target::C), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x92 as u8, Instruction::new(OpCode::SUB(Target::D), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x93 as u8, Instruction::new(OpCode::SUB(Target::E), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x94 as u8, Instruction::new(OpCode::SUB(Target::G), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x95 as u8, Instruction::new(OpCode::SUB(Target::H), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x96 as u8, Instruction::new(OpCode::SUB(Target::HL), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x97 as u8, Instruction::new(OpCode::SUB(Target::A), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));

        m.insert(0x98 as u8, Instruction::new(OpCode::SBC(Target::A, Target::B), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x99 as u8, Instruction::new(OpCode::SBC(Target::A, Target::C), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9A as u8, Instruction::new(OpCode::SBC(Target::A, Target::D), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9B as u8, Instruction::new(OpCode::SBC(Target::A, Target::E), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9C as u8, Instruction::new(OpCode::SBC(Target::A, Target::G), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9D as u8, Instruction::new(OpCode::SBC(Target::A, Target::H), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9E as u8, Instruction::new(OpCode::SBC(Target::A, Target::HL), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0x9F as u8, Instruction::new(OpCode::SBC(Target::A, Target::A), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));

        // AND
        m.insert(0xA0 as u8, Instruction::new(OpCode::AND(Target::B), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA1 as u8, Instruction::new(OpCode::AND(Target::C), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA2 as u8, Instruction::new(OpCode::AND(Target::D), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA3 as u8, Instruction::new(OpCode::AND(Target::E), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA4 as u8, Instruction::new(OpCode::AND(Target::G), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA5 as u8, Instruction::new(OpCode::AND(Target::H), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA6 as u8, Instruction::new(OpCode::AND(Target::HL), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));
        m.insert(0xA7 as u8, Instruction::new(OpCode::AND(Target::A), 1, 4, 0, vec![AFFECTED, 0, 1, 0]));

        // XOR
        m.insert(0xA8 as u8, Instruction::new(OpCode::XOR(Target::B), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xA9 as u8, Instruction::new(OpCode::XOR(Target::C), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAA as u8, Instruction::new(OpCode::XOR(Target::D), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAB as u8, Instruction::new(OpCode::XOR(Target::E), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAC as u8, Instruction::new(OpCode::XOR(Target::G), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAD as u8, Instruction::new(OpCode::XOR(Target::H), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAE as u8, Instruction::new(OpCode::XOR(Target::HL), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xAF as u8, Instruction::new(OpCode::XOR(Target::A), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));

        // OR
        m.insert(0xB0 as u8, Instruction::new(OpCode::OR(Target::B), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB1 as u8, Instruction::new(OpCode::OR(Target::C), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB2 as u8, Instruction::new(OpCode::OR(Target::D), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB3 as u8, Instruction::new(OpCode::OR(Target::E), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB4 as u8, Instruction::new(OpCode::OR(Target::G), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB5 as u8, Instruction::new(OpCode::OR(Target::H), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB6 as u8, Instruction::new(OpCode::OR(Target::HL), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));
        m.insert(0xB7 as u8, Instruction::new(OpCode::OR(Target::A), 1, 4, 0, vec![AFFECTED, 0, 0, 0]));

        // CP
        m.insert(0xB8 as u8, Instruction::new(OpCode::CP(Target::B), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xB9 as u8, Instruction::new(OpCode::CP(Target::C), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBA as u8, Instruction::new(OpCode::CP(Target::D), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBB as u8, Instruction::new(OpCode::CP(Target::E), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
        m.insert(0xBC as u8, Instruction::new(OpCode::CP(Target::G), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, AFFECTED]));
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
        m.insert(0x2C as u8, Instruction::new(OpCode::INC(Target::G), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));


        m.insert(0x33 as u8, Instruction::new(OpCode::INC(Target::SP), 1, 8, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0x34 as u8, Instruction::new(OpCode::INC(Target::HL), 1, 12, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));
        m.insert(0x3C as u8, Instruction::new(OpCode::INC(Target::A), 1, 4, 0, vec![AFFECTED, 0, AFFECTED, NOT_AFFECTED]));

        // DEC
        m.insert(0x04 as u8, Instruction::new(OpCode::DEC(Target::B), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));
        m.insert(0x0D as u8, Instruction::new(OpCode::DEC(Target::C), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));


        m.insert(0x14 as u8, Instruction::new(OpCode::DEC(Target::D), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));
        m.insert(0x1D as u8, Instruction::new(OpCode::DEC(Target::E), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));

        m.insert(0x24 as u8, Instruction::new(OpCode::DEC(Target::H), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));
        m.insert(0x2D as u8, Instruction::new(OpCode::DEC(Target::G), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));

        m.insert(0x34 as u8, Instruction::new(OpCode::DEC(Target::HL), 1, 12, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));
        m.insert(0x3D as u8, Instruction::new(OpCode::DEC(Target::A), 1, 4, 0, vec![AFFECTED, 1, AFFECTED, NOT_AFFECTED]));


        m.insert(0xC2 as u8, Instruction::new(OpCode::JUMP(Flag::NotZero), 3, 16, 12, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));
        m.insert(0xC3 as u8, Instruction::new(OpCode::JUMP_UNCONDICIONAL, 3, 16, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));



        // End of program instruction, only used for debugging
        m.insert(0xFF as u8, Instruction::new(OpCode::EndOfProgram, 0, 0, 0, vec![NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED, NOT_AFFECTED]));



        m
    };
    static ref COUNT: usize = INSTRUCTIONS.len();
}

impl Instruction {
    pub fn add_instruction(trgt: Target) -> OpCode {
        OpCode::ADD(trgt)
    }

    pub fn sub_instruction(trgt: Target) -> OpCode {
        OpCode::SUB(trgt)
    }
    pub fn inc_instruction(trgt: Target) -> OpCode {
        OpCode::INC(trgt)
    }

    pub fn from_byte(byte: u8) -> Option<&'static Instruction> {
        let i = INSTRUCTIONS.get(&byte);
        if let Some(instruction) = i {
            return Some(instruction);
        }
        None
    }

    pub fn from_opcode(_op: OpCode) -> Option<&'static Instruction> {
        for (_code, instruction) in &*INSTRUCTIONS {
            if instruction.opcode == _op {
                return Some(instruction);
            }
        }
        return None;
    }

    pub fn instruction_byte_from_opcode(_op: OpCode) -> u8 {
        for (code, instruction) in &*INSTRUCTIONS {
            if instruction.opcode == _op {
                return code.to_owned();
            }
        }

        panic!("Invalid OpCode");
    }

    pub fn byte_from_opcode(opcode: OpCode) -> Option<u8> {
        for (code, instruction) in &*INSTRUCTIONS {
            if opcode == instruction.opcode {
                return Some(code.to_owned());
            }
        }
        None
    }

    pub fn print_instruction_bytes_as_i8() {
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
