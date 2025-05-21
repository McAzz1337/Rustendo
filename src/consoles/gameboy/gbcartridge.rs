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
    pub fn new(path: String) -> GbCartridge {
        GbCartridge {
            path: path.clone(),
            data: vec![],
        }
    }

    pub fn print(&self) {
        println!("{:#?}", self.data);
    }
}

impl Cartridge for GbCartridge {
    fn load(&mut self, path: String) -> Result<(), CartridgeNotFoundError> {
        match fs::read(path.as_str()) {
            Ok(v) => {
                self.data = v;
                Ok(())
            }
            Err(e) => Err(CartridgeNotFoundError {
                what: "Failed to open file: ".to_string() + e.to_string().as_str(),
            }),
        }
    }

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
