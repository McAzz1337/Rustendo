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
use std::fs::File;
use std::io::stdin;
use std::io::Read;
use std::io::Write;

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

static RUN_FLAG: i32 = RUN_PROGRAM;

fn print_program(path: String) {
    let buffer = read_rom(path);
    let mut skip = 0;
    for i in 0..10 {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        if let Some(ins) = Instruction::look_up(buffer[i]) {
            println!("{}{:#?}", buffer[i], ins.opcode);
            skip = ins.length - 1;
        } else {
            println!("Unknown instruction: {}", buffer[i]);
        }
    }
}

fn read_rom(path: String) -> Vec<u8> {
    let mut file = File::open(&path).expect("File not found");
    let metadata = fs::metadata(&path).expect("Failed to load metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer)
        .expect("Buffer overflow when reading file");

    buffer
}

fn write_program_file(path: String) {
    let mut code: Vec<u8> = vec![];

    code.push(Instruction::byte_from_opcode(OpCode::LD(Target::A, Target::D8)).unwrap());
    code.push(0b00011000);
    code.push(Instruction::byte_from_opcode(OpCode::CB).unwrap());
    code.push(Instruction::byte_from_opcode(OpCode::SWAP(Target::A)).unwrap());

    fs::write(path, code).expect("Failed to write file");
}

fn dump_memory_to_file(path: String, buffer: &Vec<String>) {
    let mut data = "".to_string();
    for i in 0..buffer.len() {
        data += (buffer[i].clone() + "\n").as_str();
    }

    fs::write(path, data).expect("Failed to dump memory");
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

        // print_program("roms/Pokemon-Silver.gbc".to_string());
        let program = read_rom("roms/Pokemon-Silver.gbc".to_string());
        cpu.load_program(program);
        let mut memory = vec![];
        cpu.dump_memory(&mut memory);

        dump_memory_to_file("memory_dump.txt".to_string(), &memory);

        cpu.run();

        cpu.print_registers();

        println!("a = {}", cpu.get_reg_a());
        _assert_eq!(cpu.get_reg_a(), 0b10000001);
    }
}
