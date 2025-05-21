use num_traits::{NumCast, ToPrimitive};
use std::error::Error;
use std::fmt::Debug;
use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::{addressable::Addressable, readable::Readable, writeable::Writeable};

pub trait ReadDevice<A, V>: Readable<A, V> + Addressable<A> {}
pub trait WriteDevice<A, V, DV>: Writeable<A, V, DV> + Addressable<A> {}

#[derive(Debug)]
enum BusError<A>
where
    A: ToPrimitive,
{
    ReadError(A),
    WriteError(A),
}

impl<A> Display for BusError<A>
where
    A: Display + NumCast + ToPrimitive,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BusError::ReadError(addr) => format!("{}{}", "Failed to read from address: ", addr),
            BusError::WriteError(addr) => format!("{}{}", "Failed to write to address: ", addr),
        };
        write!(f, "{}", s)
    }
}

impl<A> Error for BusError<A> where A: Debug + Display + NumCast + ToPrimitive {}

pub struct Bus<A, V, DV> {
    readables: Vec<Rc<RefCell<dyn ReadDevice<A, V>>>>,
    writeables: Vec<Rc<RefCell<dyn WriteDevice<A, V, DV>>>>,
}

impl<A, V, DV> Bus<A, V, DV> {
    pub fn new() -> Self {
        Bus {
            readables: vec![],
            writeables: vec![],
        }
    }

    pub fn connect_readable(&mut self, readable: Rc<RefCell<dyn ReadDevice<A, V>>>) {
        self.readables.push(readable);
    }

    pub fn connect_writeable(&mut self, writeable: Rc<RefCell<dyn WriteDevice<A, V, DV>>>) {
        self.writeables.push(writeable);
    }
}

impl<A, V, DV> Readable<A, V> for Bus<A, V, DV>
where
    A: NumCast + ToPrimitive + Display + Debug + 'static + Copy + Clone,
{
    fn read(&self, address: A) -> Result<V, Box<dyn Error>> {
        if let Some(readable) = self.readables.iter().find(|r| r.borrow().in_range(address)) {
            readable.borrow().read(address)
        } else {
            Err(Box::new(BusError::ReadError::<A>(address)))
        }
    }
}

impl<A, V, DV> Writeable<A, V, DV> for Bus<A, V, DV>
where
    A: NumCast + ToPrimitive + Display + Debug + 'static + Copy + Clone,
{
    fn write(&mut self, address: A, data: V) -> Result<(), Box<dyn Error>> {
        if let Some(writeable) = self
            .writeables
            .iter()
            .find(|w| w.borrow().in_range(address))
        {
            writeable.as_ref().borrow_mut().write(address, data);
            Ok(())
        } else {
            Err(Box::new(BusError::WriteError::<A>(address)))
        }
    }

    fn write_16(&mut self, address: A, data: DV) -> Result<(), Box<dyn Error>> {
        if let Some(writeable) = self
            .writeables
            .iter()
            .find(|w| w.borrow().in_range(address))
        {
            writeable.as_ref().borrow_mut().write_16(address, data);
            Ok(())
        } else {
            Err(Box::new(BusError::WriteError::<A>(address)))
        }
    }
}
