use std::error::Error;
use std::fs;

use crate::consoles::addressable::Addressable;
use crate::consoles::bus::ReadDevice;
use crate::consoles::readable::Readable;

use super::super::cartridge::Cartridge;
use super::super::cartridge::CartridgeNotFoundError;
use super::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct GbCartridge {
    path: String,
    data: Vec<u8>,
}

impl GbCartridge {
    pub fn new(path: String) -> Result<GbCartridge, Box<dyn Error>> {
        match fs::read(path.as_str()) {
            Ok(v) => Ok(GbCartridge { path, data: v }),
            Err(e) => Err(Box::new(CartridgeNotFoundError {
                what: format!("{}{}", "Failed to open file: ", e.to_string()),
            })),
        }
    }

    pub fn print(&self) {
        println!("{:#?}", self.data);
    }
}

impl Cartridge for GbCartridge {
    fn dump(&self) -> String {
        let mut dump = String::new();
        let mut data_words = 0;

        for i in self.data.iter() {
            if data_words > 0 {
                dump = dump + i.to_string().as_str();

                if data_words > 1 {
                    dump = dump + ", ";
                } else {
                    dump = dump + "\n";
                }

                data_words -= 1;
            } else if let Some(instruction) = Instruction::look_up(*i) {
                dump = dump + Instruction::mnemonic_as_string(i).as_str() + "\t";
                data_words = (instruction.length as i8 - 1).max(0);

                if data_words == 0 {
                    dump = dump + "\n";
                }
            }
        }

        dump
    }

    fn dump_raw(&self) -> String {
        self.data
            .iter()
            .map(u8::to_string)
            .fold(String::new(), |a, b| a + b.as_str())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ReadDevice<u16, u8> for GbCartridge {}

impl Readable<u16, u8> for GbCartridge {
    fn read(&self, address: u16) -> Result<u8, Box<dyn std::error::Error>> {
        Ok(self.data[address as usize])
    }
}

impl Addressable<u16> for GbCartridge {
    fn in_range(&self, address: u16) -> bool {
        todo!()
    }
}
