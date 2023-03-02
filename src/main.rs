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

use crate::instruction::INSTRUCTIONS;

use std::env;
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

static CHECK_INSTRUCTION_DEFINITION_COMPLETENES: i32 = 0b0001;
static PRINT_OPCODES: i32 = 0b0010;
static CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES: i32 = 0b0100;
static RUN_PROGRAM: i32 = 0b1000;

static RUN_FLAG: i32 = CHECK_INSTRUCTION_DEFINITION_COMPLETENES
    | CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES
    | RUN_PROGRAM;

fn read_program_file(path: String, cpu: &mut Cpu) {
    let code = fs::read(path).expect("Failed to read file");

    let mut has_data = false;
    for i in 0..code.len() {
        if has_data {
            println!("{}", code[i]);
            has_data = false;
            cpu.write_to_memory(cpu.get_pc() - i as u16, code[i]);
            continue;
        }
        if let Some(instruction) = Instruction::look_up(code[i]) {
            has_data = instruction.length > 1;
            println!("{}: {:#?}", i, instruction);
        }
        cpu.write_to_memory(cpu.get_pc() - i as u16, code[i]);
    }
    cpu.get_memory().print_memory_mnemonics();
}

fn write_program_file(path: String) {
    let mut code: Vec<u8> = vec![];

    code.push(Instruction::byte_from_opcode(OpCode::LD(Target::A, Target::D8)).unwrap());
    code.push(0b00011000);
    code.push(Instruction::byte_from_opcode(OpCode::CB).unwrap());
    code.push(Instruction::byte_from_opcode(OpCode::SWAP(Target::A)).unwrap());

    fs::write(path, code).expect("Failed to write file");
}

fn main() {
    let mut cpu = Cpu::new();
    if RUN_FLAG & CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES != 0 {
        println!("Checking instruction implementation completeness");
        // env::set_var("RUST_BACKTRACE", "1");
        cpu.zero_memory();

        for i in 0..=0xFF {
            if let Some(instruction) = Instruction::fetch(i, false) {
                // println!("Executing instruction: [{:#x}] {:#?}", i, instruction);
                cpu.execute(instruction);
                cpu.reset_registers();
            }
        }
    }

    if RUN_FLAG & CHECK_INSTRUCTION_DEFINITION_COMPLETENES != 0 {
        println!("Checking instruction declaration completeness");
        Instruction::test_instruction_completeness();
    }

    if RUN_FLAG & PRINT_OPCODES != 0 {
        Instruction::print_instruction_bytes_as_char();
    }

    if RUN_FLAG & RUN_PROGRAM != 0 {
        println!("Starting program");
        cpu.set_memory_to_end_of_program();

        cpu.power_up();

        // write_program_file("program.bin".to_string());
        read_program_file("program.bin".to_string(), &mut cpu);

        cpu.run();

        cpu.print_registers();

        println!("a = {}", cpu.get_reg_a());
        _assert_eq!(cpu.get_reg_a(), 0b10000001);
    }
}
