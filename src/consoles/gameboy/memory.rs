use std::error::Error;

use super::instruction::{Instruction, INSTRUCTIONS};
use super::opcode::OpCode::EndOfProgram;
use crate::consoles::addressable::Addressable;
use crate::consoles::bus::{ReadDevice, WriteDevice};
use crate::consoles::readable::Readable;
use crate::consoles::writeable::Writeable;
use crate::utils::conversion;

pub const INTERRPUT_ENABLE: u16 = 0xFFFF;
pub const INTERNAL_RAM: u16 = 0xFF80;
pub const EMPTY_BUT_UNUSABLE_FOR_IO: u16 = 0xFF4C;
pub const IO_PORTS: u16 = 0xFF00;
pub const EMPTY_BUT_UNUSABLE_FOR_IO_2: u16 = 0xFEA0;
pub const SPRITE_ATTRIB: u16 = 0xFE00;
pub const ECHO_OF_INTERNAL_RAM: u16 = 0xE000;
pub const RAM: u16 = 0xC000;
pub const SWITCHABLE_RAM: u16 = 0xA000;
pub const VIDEO_RAM: u16 = 0x8000;
pub const SWITCHABLE_ROM: u16 = 0x4000;
pub const ROM: u16 = 0x0000;

const RAM_ECHO_OFFSET: u16 = ECHO_OF_INTERNAL_RAM - RAM;

pub const TILE_PATTERN_1: u16 = 0x8000;
pub const TILE_PATTERN_2: u16 = 0x9000;

lazy_static! {
    static ref NINTENDO_SPLASH_SCREEN: Vec<u8> = vec![
        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00,
        0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD,
        0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB,
        0xB9, 0x33, 0x3E,
    ];
}
pub struct Memory {
    size: usize,
    memory: [u8; 0x10000],
}
impl Memory {
    pub fn new() -> Memory {
        let mut memory = Memory {
            size: 0x10000,
            memory: [0; 0x10000],
        };

        for i in 0..memory.size {
            let _ = memory.write(
                i as u16,
                Instruction::byte_from_opcode(EndOfProgram).unwrap(),
            );
        }

        for i in 0..NINTENDO_SPLASH_SCREEN.len() {
            let _ = memory.write(i as u16 + 104, NINTENDO_SPLASH_SCREEN[i]);
        }

        memory
    }

    pub fn read_as_binary_string(&self, address: u16) -> String {
        conversion::u8_as_bit_string(self.memory[address as usize])
    }

    pub fn read_as_hex_strign(&self, address: u16) -> String {
        conversion::u8_as_hex_string(self.memory[address as usize])
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn print_memory_readable(&self) {
        let mut data_words = 0;

        for i in 0..self.size {
            if data_words > 0 {
                print!("{}\t", self.memory[i]);
                if data_words == 1 {
                    println!();
                }
                data_words -= 1;
            } else if let Some(ins) = Instruction::look_up(self.memory[i]) {
                data_words = ins.length - 1;

                if data_words > 0 {
                    print!("{}\t", Instruction::mnemonic_as_string(&self.memory[i]));
                } else {
                    println!("{}", Instruction::mnemonic_as_string(&self.memory[i]));
                }
            }
        }
    }

    pub fn dump_memory(&self, buffer: &mut Vec<String>) {
        let mut data_words = 0;
        let mut line = "".to_string();
        for i in 0..self.size {
            if data_words > 0 {
                line += self.memory[i].to_string().as_str();
                if data_words == 1 {
                    buffer.push(line.clone());
                }
                data_words -= 1;
            } else if let Some(ins) = Instruction::look_up(self.memory[i]) {
                data_words = ins.length;

                if data_words > 0 {
                    data_words -= 1;
                    line = Instruction::mnemonic_as_string(&self.memory[i]) + "\t";
                } else {
                    buffer.push(Instruction::mnemonic_as_string(&self.memory[i]));
                }
            }
        }
    }

    pub fn print(&self) {
        println!("MEMORY:");
        let mut has_data = false;
        for i in 0..self.size {
            if self.memory[i] == Instruction::byte_from_opcode(EndOfProgram).unwrap() {
                continue;
            }
            if !has_data && INSTRUCTIONS.contains_key(&(i as u8)) {
                println!("[{:#x}] :\t{:#x}", i, self.memory[i]);
                if INSTRUCTIONS.get(&(i as u8)).unwrap().length > 1 {
                    has_data = true;
                }
            } else {
                println!("[{:#x}] :\t{}", i, self.memory[i]);
                has_data = false;
            }
        }
        println!("--------------------------------");
    }

    pub fn print_memory_mnemonics(&self) {
        println!("MEMORY:");
        let mut data_words = 0;
        for i in 0..self.size {
            if data_words > 0 {
                println!("{:#x}:\t{}", i, self.memory[i]);
                data_words -= 1;
            }
            if let Some(instruction) = Instruction::look_up(self.memory[i]) {
                if instruction.length > 0 {
                    data_words = instruction.length - 1;
                } else {
                    data_words = 0;
                }
                println!(
                    "{:#x}:\t{}\t{:#x}",
                    i,
                    Instruction::mnemonic_as_string(&self.memory[i]),
                    self.memory[i]
                );
            } else {
                println!("{:#x}:\t{}", i, self.memory[i]);
                data_words -= 1;
            }
        }
    }
}

impl Readable for Memory {
    fn read(&self, address: u16) -> Result<u8, Box<dyn std::error::Error>> {
        Ok(self.memory[address as usize])
    }
}

impl ReadDevice for Memory {}

impl Writeable for Memory {
    fn write(&mut self, address: u16, data: u8) -> Result<(), Box<dyn Error>> {
        self.memory[address as usize] = data;
        // self.memory[(address + RAM_ECHO_OFFSET) as usize] = data;
        Ok(())
    }

    fn write_16(&mut self, address: u16, data: u16) -> Result<(), Box<dyn Error>> {
        let upper = (data >> 8) as u8;
        let lower = (data & 0xFF) as u8;
        self.memory[address as usize] = upper;
        self.memory[(address + 1) as usize] = lower;
        Ok(())
    }
}

impl WriteDevice for Memory {}

impl Addressable for Memory {
    fn in_range(&self, address: u16) -> bool {
        (0..=u16::MAX).contains(&address)
    }
}

#[test]
fn test_memory() {
    let mut mem = Memory::new();
    let _ = mem.write(RAM, 100);
    assert!(mem.read(RAM).unwrap() == 100);
    assert!(mem.read(ECHO_OF_INTERNAL_RAM).unwrap() == 100);
    let _ = mem.write(RAM + 40, 10);
    assert!(mem.read(RAM + 40).unwrap() == 10);
    assert!(mem.read(ECHO_OF_INTERNAL_RAM + 40).unwrap() == 10);
}
