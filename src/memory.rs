use crate::instruction::Instruction;
use crate::utils;

use crate::instruction::INSTRUCTIONS;

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
            memory.write_byte(
                i as u16,
                Instruction::byte_from_opcode(crate::instruction::OpCode::EndOfProgram).unwrap(),
            );
        }

        for i in 0..NINTENDO_SPLASH_SCREEN.len() {
            memory.write_byte(i as u16 + 104, NINTENDO_SPLASH_SCREEN[i as usize]);
        }

        memory
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
        if (address as usize + RAM_ECHO_OFFSET as usize) < self.size as usize {
            self.memory[(address + RAM_ECHO_OFFSET) as usize] = byte;
        }
    }

    pub fn write_2_bytes(&mut self, address: u16, bytes: u16) {
        self.memory[address as usize] = (bytes & 0b11111111) as u8;
        self.memory[address as usize + 1] = ((bytes & 0b1111111100000000) >> 8) as u8;
        if ((address + 1 + RAM_ECHO_OFFSET) as usize) < self.size {
            self.memory[(address + RAM_ECHO_OFFSET) as usize] = (bytes & 0b11111111) as u8;
            self.memory[(address + RAM_ECHO_OFFSET) as usize + 1] =
                ((bytes & 0b1111111100000000) >> 8) as u8;
        }
    }

    pub fn get_pointer(&mut self, address: u16) -> *mut u8 {
        &mut self.memory[address as usize]
    }

    pub fn read_as_binary_string(&self, address: u16) -> String {
        utils::as_bit_string(self.memory[address as usize])
    }

    pub fn read_as_hex_strign(&self, address: u16) -> String {
        utils::as_hex_string(self.memory[address as usize])
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn print(&mut self) {
        println!("MEMORY:");
        let mut has_data = false;
        for i in 0..self.size {
            if self.memory[i]
                == Instruction::byte_from_opcode(crate::instruction::OpCode::EndOfProgram).unwrap()
            {
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
                data_words = data_words - 1;
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
                data_words = data_words - 1;
            }
        }
    }
}

#[test]
fn test_memory() {
    let mut mem = Memory::new();
    mem.write_byte(RAM, 100);
    assert!(mem.read_byte(RAM) == 100);
    assert!(mem.read_byte(ECHO_OF_INTERNAL_RAM) == 100);
    mem.write_byte(RAM + 40, 10);
    assert!(mem.read_byte(RAM + 40) == 10);
    assert!(mem.read_byte(ECHO_OF_INTERNAL_RAM + 40) == 10);
}
