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

static CHECK_INSTRUCTION_DEFINITION_COMPLETENES: bool = true;
static PRINT_OPCODES: bool = false;
static CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES: bool = false;

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
        if let Some(instruction) = Instruction::look_up(code[i]) {
            has_data = instruction.length > 1;
            println!("{}: {:#?}", i, instruction);
        }
        cpu.write_to_memory(i as u16, code[i]);
    }
}

fn write_program_file(path: String) {
    let mut code: Vec<u8> = vec![];

    code.push(Instruction::byte_from_opcode(OpCode::LD(Target::A, Target::D8)).unwrap());
    code.push(0b00011000);
    code.push(Instruction::byte_from_opcode(OpCode::PREFIX).unwrap());
    code.push(Instruction::byte_from_opcode(OpCode::SWAP(Target::A)).unwrap());

    fs::write(path, code).expect("Failed to write file");
}

fn main() {
    if CHECK_INSTRUCTION_IMPLEMNETATION_COMPLETENES {
        // env::set_var("RUST_BACKTRACE", "1");
        let mut cpu = Cpu::new();
        cpu.zero_memory();

        for i in 0..=0xFF {
            if let Some(instruction) = Instruction::fetch(i, false) {
                // println!("Executing instruction: [{:#x}] {:#?}", i, instruction);
                cpu.execute(instruction);
                cpu.reset_registers();
            }
        }

        // for i in INSTRUCTIONS.clone() {
        //     println!("Executing instruction: [{:#x}] {:#?}", &i.0, &i.1);
        //     cpu.execute(&i.1);
        //     cpu.reset_registers();
        // }
        return;
    }

    if CHECK_INSTRUCTION_DEFINITION_COMPLETENES {
        Instruction::test_instruction_completeness();
        return;
    }

    if PRINT_OPCODES {
        Instruction::print_instruction_bytes_as_char();
    }

    let mut cpu = Cpu::new();

    // write_program(&mut cpu);
    // write_program_file("program.bin".to_string());
    read_program_file("program.bin".to_string(), &mut cpu);

    cpu.run();

    cpu.print_registers();

    println!("a = {}", cpu.get_reg_a());
    _assert_eq!(cpu.get_reg_a(), 0b10000001);
}
