use crate::target::Target;
use crate::cpu::registers::Flag;

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