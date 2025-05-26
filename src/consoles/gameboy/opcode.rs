use std::fmt::Display;

use super::registers::Flag;
use super::target::Target;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    ADC(Target),
    ADD(Target),
    ADD16(Target, Target),
    AND(Target),
    BIT(u8, Target),
    CALL(Flag),
    CALL_UC,
    CB,         //Prefix
    CCF,        //Toggle value of carry flag
    CP(Target), // like sub but result not stored back into a
    CPL,
    DDA,
    DEC(Target),
    DEC16(Target),
    DisableInterrupt,
    EnableInterrupt,
    HALT,
    INC(Target),
    INC16(Target),
    JUMP(Flag),
    JP,
    JP_HL,
    JR(Flag),
    JRUC, // special case for JR r8 instruction
    LD(Target, Target),
    LDH(Target, Target),
    LDA,
    NOP,
    OR(Target),
    POP(Target),
    PUSH(Target),
    RES(u8, Target),
    RET_UC,
    RET(Flag),
    RETI,
    RL(Target), //rotate left specific register through carry flag
    RLA,
    RLC(Target), // rotate left specific register not through carry flag
    RLCA,
    RCA,
    RR(Target), //rotate right specific register through carry flag
    RRA,
    RRC(Target), // rotate right specific register not through carry flag
    RRCA,
    RRLA, // totate left a rg not through carry flag
    RST(u16),
    SBC(Target),
    SCF, // set carry flag to true
    SET(u8, Target),
    SLA(Target), // shift left by 1
    SRA(Target), // shift right by 1
    SRL(Target), // right shift of specific regiser
    STOP,
    SUB(Target),
    SWAP(Target), //swap upper and lower nibble

    XOR(Target),
    EndOfProgram,

    STORE(Target, Target), // These instructions do not exist for the game boy and are only used as convenience instructions
    STORE16(Target, Target), // First arg dst address 16 bit, seconds arg src direct value 8 / 16 bit
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ADC(_) => "ADC",
            Self::ADD(_) => "ADD",
            Self::ADD16(_, _) => "ADD16",
            Self::AND(_) => "AND",
            Self::BIT(_, _) => "BIT",
            Self::CALL(_) => "CALL",
            Self::CALL_UC => "CALL_UC",
            Self::CB => "CB",
            Self::CCF => "CCF",
            Self::CP(_) => "CP",
            Self::CPL => "CPL",
            Self::DDA => "DDA",
            Self::DEC(_) => "DEC",
            Self::DEC16(_) => "DEC16",
            Self::DisableInterrupt => "DisableInterrupt",
            Self::EnableInterrupt => "EnableInterrupt",
            Self::HALT => "HALT",
            Self::INC(_) => "INC",
            Self::INC16(_) => "INC16",
            Self::JUMP(_) => "JUMP",
            Self::JP => "JP",
            Self::JP_HL => "JP_HL",
            Self::JR(_) => "JR",
            Self::JRUC => "JRUC",
            Self::LD(_, _) => "LD",
            Self::LDH(_, _) => "LDH",
            Self::LDA => "LDA",
            Self::NOP => "NOP",
            Self::OR(_) => "OR",
            Self::POP(_) => "POP",
            Self::PUSH(_) => "PUSH",
            Self::RES(_, _) => "RES",
            Self::RET_UC => "RET_UC",
            Self::RET(_) => "RET",
            Self::RETI => "RETI",
            Self::RL(_) => "RL",
            Self::RLA => "RLA",
            Self::RLC(_) => "RLC",
            Self::RLCA => "RLCA",
            Self::RCA => "RCA",
            Self::RR(_) => "RR",
            Self::RRA => "RRA",
            Self::RRC(_) => "RRC",
            Self::RRCA => "RRCA",
            Self::RRLA => "RRLA",
            Self::RST(_) => "RST",
            Self::SBC(_) => "SBC",
            Self::SCF => "SCF",
            Self::SET(_, _) => "SET",
            Self::SLA(_) => "SLA",
            Self::SRA(_) => "SRA",
            Self::SRL(_) => "SRL",
            Self::STOP => "STOP",
            Self::SUB(_) => "SUB",
            Self::SWAP(_) => "SWAP",
            Self::XOR(_) => "XOR",
            Self::EndOfProgram => "EndOfProgram",
            Self::STORE(_, _) => "STORE",
            Self::STORE16(_, _) => "STORE16",
        };

        write!(f, "{s}")
    }
}
