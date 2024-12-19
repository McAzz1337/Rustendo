use std::error::Error;
use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::{addressable::Addressable, readable::Readable, writeable::Writeable};

pub trait ReadDevice: Readable + Addressable {}
pub trait WriteDevice: Writeable + Addressable {}

#[derive(Debug)]
enum BusError {
    AddressError(u16),
    ReadError,
    WriteError,
}

impl Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BusError::AddressError(addr) => &format!("{}{}", "No such address found: ", addr),
            BusError::ReadError => "Failed to read from address",
            BusError::WriteError => "Failed to write to address",
        };
        write!(f, "{}", s)
    }
}

impl Error for BusError {}

pub struct Bus {
    readables: Vec<Rc<RefCell<dyn ReadDevice>>>,
    writeables: Vec<Rc<RefCell<dyn WriteDevice>>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            readables: vec![],
            writeables: vec![],
        }
    }

    pub fn connect_readable(&mut self, readable: Rc<RefCell<dyn ReadDevice>>) {
        self.readables.push(readable);
    }

    pub fn connect_writeable(&mut self, writeable: Rc<RefCell<dyn WriteDevice>>) {
        self.writeables.push(writeable);
    }
}

impl Readable for Bus {
    fn read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        if let Some(readable) = self.readables.iter().find(|r| r.borrow().in_range(address)) {
            readable.borrow().read(address)
        } else {
            Err(Box::new(BusError::AddressError(address)))
        }
    }
}

impl Writeable for Bus {
    fn write(&mut self, address: u16, data: u8) -> Result<(), Box<dyn Error>> {
        if let Some(writeable) = self
            .writeables
            .iter()
            .find(|w| w.borrow().in_range(address))
        {
            writeable.as_ref().borrow_mut().write(address, data);
            Ok(())
        } else {
            Err(Box::new(BusError::WriteError))
        }
    }

    fn write_16(&mut self, address: u16, data: u16) -> Result<(), Box<dyn Error>> {
        if let Some(writeable) = self
            .writeables
            .iter()
            .find(|w| w.borrow().in_range(address))
        {
            writeable.as_ref().borrow_mut().write_16(address, data);
            Ok(())
        } else {
            Err(Box::new(BusError::WriteError))
        }
    }
}
