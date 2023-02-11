#[macro_use]
extern crate lazy_static;

pub mod cpu;
pub mod instruction;
pub mod memory;
pub mod registers;
pub mod utils;

use crate::cpu::Cpu;
use crate::instruction::OpCode;
use crate::instruction::Target;
use crate::instruction::*;
use crate::memory::Memory;
use crate::registers::Flag;
use crate::registers::Registers;

macro_rules! _assert {
    ($a: expr, $b: expr, $c: expr) => {
        if (!$a) {
            eprintln!("expected: {}\tactual: {}", $b, $c);
        }
        assert!($a);
    };
}

static CHECK_INSTRUCTION_COMPLETENESS: bool = false;

fn write_program(cpu: &mut Cpu) {
    let code = vec![
        Instruction::instruction_byte_from_opcode(OpCode::LDR(Target::A, Target::D8)),
        100,
        Instruction::instruction_byte_from_opcode(OpCode::LDR(Target::B, Target::D8)),
        50,
        Instruction::instruction_byte_from_opcode(OpCode::ADD(Target::B)),
        Instruction::instruction_byte_from_opcode(OpCode::LDR(Target::B, Target::A)),
        Instruction::instruction_byte_from_opcode(OpCode::LDR(Target::A, Target::D8)),
        100,
        Instruction::instruction_byte_from_opcode(OpCode::ADD(Target::B)),
    ];

    let mut address = 0;
    for byte in code {
        // if let Some(_instruction) = Instruction::from_byte(byte) {
        //     println!("op: {}", utils::as_hex_string(byte));
        // } else {
        //     println!("{}", byte);
        // }
        cpu.write_to_memory(address, byte);
        address = address + 1;
    }
}

fn main() {
    if CHECK_INSTRUCTION_COMPLETENESS {
        Instruction::test_instruction_completeness();
    }

    // Instruction::print_instruction_bytes_as_i8();
    let mut cpu = Cpu::new();

    write_program(&mut cpu);

    cpu.run();

    _assert!(cpu.get_reg_a() == 250, 250, cpu.get_reg_a());
}
