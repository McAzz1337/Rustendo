use std::error::Error;

pub trait Writeable<A, V, DV> {
    fn write(&mut self, address: A, data: V) -> Result<(), Box<dyn Error>>;
    fn write_16(&mut self, address: A, data: DV) -> Result<(), Box<dyn Error>>;
}
