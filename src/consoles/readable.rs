use std::error::Error;

pub trait Readable {
    fn read(&self, address: u16) -> Result<u8, Box<dyn Error>>;
}
