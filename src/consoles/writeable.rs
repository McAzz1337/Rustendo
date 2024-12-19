use std::error::Error;

pub trait Writeable {
    fn write(&mut self, address: u16, data: u8) -> Result<(), Box<dyn Error>>;
    fn write_16(&mut self, address: u16, data: u16) -> Result<(), Box<dyn Error>>;
}
