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

use std::fs;

macro_rules! _assert_eq {
    ($a: expr, $b: expr) => {
        if ($a != $b) {
            eprintln!(
                "Unexpected value [{}]\n\texpected: {}\tactual: {}",
                stringify!($a),
                $b,
                $a
            );
        }
    };
}

macro_rules! _assert {
    ($a: expr, $b: expr, $c: expr) => {
        if (!$a) {
            eprintln!("expected: {}\tactual: {}", $b, $c);
        }
        assert!($a);
    };
}

static CHECK_INSTRUCTION_COMPLETENESS: bool = false;
static PRINT_OPCODES: bool = false;

fn read_program_file(path: String, cpu: &mut Cpu) {
    let code = fs::read(path).expect("Failed to read file");

    let mut has_data = false;
    for i in 0..code.len() {
        if has_data {
            println!("{}", code[i]);
            has_data = false;
            cpu.write_to_memory(i as u16, code[i]);
            continue;
        }
        if let Some(instruction) = Instruction::from_byte(code[i]) {
            has_data = instruction.length > 1;
            println!("{}: {:#?}", i, instruction);
        }
        cpu.write_to_memory(i as u16, code[i]);
    }
}

fn write_program_file(path: String) {
    let mut code: Vec<u8> = vec![];

    code.push(Instruction::byte_from_opcode(OpCode::LD(Target::A, Target::D8)).unwrap());
    code.push(100);
    code.push(Instruction::byte_from_opcode(OpCode::LD(Target::B, Target::D8)).unwrap());
    code.push(50);
    code.push(Instruction::byte_from_opcode(OpCode::ADD(Target::B)).unwrap());
    code.push(Instruction::byte_from_opcode(OpCode::LD(Target::B, Target::A)).unwrap());
    code.push(Instruction::byte_from_opcode(OpCode::JUMP(Flag::NotZero)).unwrap());
    code.push(20);

    let start = code.len();
    let end = 20;
    for _i in start..end {
        code.push(Instruction::byte_from_opcode(OpCode::NOP).unwrap());
    }

    code.push(Instruction::byte_from_opcode(OpCode::LD(Target::A, Target::D8)).unwrap());
    code.push(80);

    fs::write(path, code).expect("Failed to write file");
}

fn write_program(cpu: &mut Cpu) {
    let code = vec![
        Instruction::instruction_byte_from_opcode(OpCode::LD(Target::A, Target::D8)),
        100,
        Instruction::instruction_byte_from_opcode(OpCode::LD(Target::B, Target::D8)),
        50,
        Instruction::instruction_byte_from_opcode(OpCode::ADD(Target::B)),
        Instruction::instruction_byte_from_opcode(OpCode::LD(Target::B, Target::A)),
        Instruction::instruction_byte_from_opcode(OpCode::LD(Target::A, Target::D8)),
        100,
        Instruction::instruction_byte_from_opcode(OpCode::ADD(Target::B)),
    ];

    let mut address = 0;
    for byte in code {
        cpu.write_to_memory(address, byte);
        address = address + 1;
    }
}

fn main() {
    if CHECK_INSTRUCTION_COMPLETENESS {
        Instruction::test_instruction_completeness();
    }

    if PRINT_OPCODES {
        Instruction::print_instruction_bytes_as_i8();
    }

    let mut cpu = Cpu::new();

    // write_program(&mut cpu);
    write_program_file("program.bin".to_string());
    read_program_file("program.bin".to_string(), &mut cpu);

    cpu.run();

    println!("a = {}", cpu.get_reg_a());
    _assert_eq!(cpu.get_reg_a(), 80);
}
