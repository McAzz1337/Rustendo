use std::ops::RangeInclusive;

use crate::{shift_left, shift_right};

use super::{
    addressable::Addressable,
    bus::{ReadDevice, WriteDevice},
    memory_map::gameboy::ROM_BANK_00,
    readable::Readable,
    writeable::Writeable,
};

#[derive(Debug, Clone)]
pub struct FakeCartridge {
    path: String,
    data: Vec<u8>,
    address_range: RangeInclusive<usize>,
}

impl FakeCartridge {
    pub fn new() -> FakeCartridge {
        let data = ROM_BANK_00.map(|_| 0).collect();
        FakeCartridge {
            path: "fake:cartridge".to_string(),
            data,
            address_range: (0..=0),
        }
    }

    fn dump(&self) -> String {
        self.dump_raw()
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

impl ReadDevice<u16, u8> for FakeCartridge {}

impl Readable<u16, u8> for FakeCartridge {
    fn read(&self, address: u16) -> Result<u8, Box<dyn std::error::Error>> {
        Ok(self.data[address as usize])
    }
}

impl WriteDevice<u16, u8, u16> for FakeCartridge {}

impl Writeable<u16, u8, u16> for FakeCartridge {
    fn write(&mut self, address: u16, data: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.data[address as usize] = data;
        Ok(())
    }

    fn write_16(&mut self, address: u16, data: u16) -> Result<(), Box<dyn std::error::Error>> {
        let upper = shift_right!(data, 8, u8);
        let lower = (data & 0xFF) as u8;
        self.data[address as usize] = lower;
        self.data[(address + 1) as usize] = upper;
        Ok(())
    }
}

impl Addressable<u16> for FakeCartridge {
    fn assign_address_range(&mut self, range: std::ops::RangeInclusive<usize>) {
        self.address_range = range;
    }

    fn in_range(&self, address: u16) -> bool {
        self.address_range.contains(&(address as usize))
    }
}
