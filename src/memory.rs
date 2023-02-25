use crate::instruction::Instruction;
use crate::utils;

pub struct Memory {
    memory: [u8; 0xFFFF],
}
impl Memory {
    pub fn new() -> Memory {
        let mut memory = Memory {
            memory: [0; 0xFFFF],
        };

        for i in 0..0xFF {
            memory.write_byte(
                i,
                Instruction::byte_from_opcode(crate::instruction::OpCode::EndOfProgram).unwrap(),
            );
        }

        memory
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
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
}

#[test]
fn test_memory() {
    let mut mem = Memory::new();
    mem.write_byte(0, 100);
    assert!(mem.read_byte(0) == 100);
    mem.write_byte(40, 10);
    assert!(mem.read_byte(40) == 10);
}
