use std::error::Error;
use std::fmt::{Debug, Display, LowerHex};
use std::marker::PhantomData;
use std::ops::{BitAnd, Shr};

use num_traits::{AsPrimitive, FromPrimitive, NumCast, ToPrimitive};

use crate::consoles::addressable::Addressable;
use crate::consoles::bus::{ReadDevice, WriteDevice};
use crate::consoles::readable::Readable;
use crate::consoles::writeable::Writeable;
#[allow(unused_imports)]
use crate::utils::conversion::u16_to_u8;

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

#[allow(dead_code)]
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

#[derive(Debug)]
enum MemoryError<A> {
    ReadError(A),
    WriteError(A),
}

impl<A> Display for MemoryError<A>
where
    A: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ReadError(addr) => &format!("Failed to read from address: {:?}", addr),
            Self::WriteError(addr) => &format!("Failed to write to address: {:?}", addr),
        };
        write!(f, "{s}")
    }
}

impl<A> Error for MemoryError<A> where A: Debug {}

pub struct Memory<A, V, DV> {
    address_type: PhantomData<A>,
    d_value_type: PhantomData<DV>,
    size: usize,
    memory: [V; 0x10000],
    conversion: fn(DV) -> Option<(V, V)>,
}

impl<A, V, DV> Memory<A, V, DV>
where
    A: NumCast + Copy + Clone + Debug + 'static,
    V: LowerHex
        + PartialEq
        + Display
        + Default
        + Copy
        + FromPrimitive
        + NumCast
        + ToPrimitive
        + AsPrimitive<V>,
    DV: NumCast + Shr<i32> + BitAnd<u16>,
{
    pub fn new(
        conversion: fn(DV) -> Option<(V, V)>,
        get_default_value: Option<Box<dyn Fn() -> V>>,
    ) -> Self {
        let mut memory = Memory::<A, V, DV> {
            address_type: PhantomData,
            d_value_type: PhantomData,
            size: 0x10000,
            memory: [V::default(); 0x10000],
            conversion,
        };

        if let Some(get_default_value) = get_default_value {
            (0..memory.size).into_iter().for_each(|i| {
                let _ = <Memory<A, V, DV> as Writeable<A, V, DV>>::write(
                    &mut memory,
                    NumCast::from(i as u16).unwrap(),
                    NumCast::from(get_default_value()).unwrap(),
                );
            });
        }

        (0..NINTENDO_SPLASH_SCREEN.len()).into_iter().for_each(|i| {
            let _ = memory.write(
                NumCast::from(i as u16 + 104).unwrap(),
                NumCast::from(NINTENDO_SPLASH_SCREEN[i]).unwrap(),
            );
        });

        memory
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn as_hex_dump(&self) -> Vec<String> {
        self.memory
            .iter()
            .flat_map(|x| x.to_u8())
            .map(|x| format!("{:#x}", x))
            .collect()
    }
}

impl<A, V, DV> Readable<A, V> for Memory<A, V, DV>
where
    A: NumCast + AsPrimitive<A> + ToPrimitive + Debug,
    V: Copy,
{
    fn read(&self, address: A) -> Result<V, Box<dyn Error>> {
        match address.to_usize() {
            Some(index) => Ok(self.memory[index]),
            None => Err(Box::new(MemoryError::ReadError::<A>(address))),
        }
    }
}

impl<A, V, DV> ReadDevice<A, V> for Memory<A, V, DV>
where
    A: NumCast + AsPrimitive<A> + Clone + Copy + Debug,
    V: Copy + Clone,
{
}

impl<A, V, DV> Writeable<A, V, DV> for Memory<A, V, DV>
where
    A: NumCast + Clone + Copy + Debug + 'static,
    V: NumCast,
    DV: NumCast + Shr<i32> + BitAnd<u16>,
{
    fn write(&mut self, address: A, data: V) -> Result<(), Box<dyn Error>> {
        match address.to_usize() {
            Some(index) => {
                self.memory[index] = data;
                Ok(())
            }
            None => Err(Box::new(MemoryError::WriteError::<A>(address))),
        }
    }

    fn write_16(&mut self, address: A, data: DV) -> Result<(), Box<dyn Error>> {
        match address.to_usize() {
            Some(index) => match (self.conversion)(data) {
                Some((upper, lower)) => {
                    self.memory[index] = NumCast::from(upper).unwrap();
                    self.memory[index + 1] = NumCast::from(lower).unwrap();
                    Ok(())
                }
                None => todo!(),
            },
            None => todo!(),
        }
    }
}

impl<A, V, DV> WriteDevice<A, V, DV> for Memory<A, V, DV>
where
    A: NumCast + Copy + Clone + Debug + 'static,
    V: NumCast,
    DV: NumCast + Shr<i32> + BitAnd<u16>,
{
}

impl<A, V, DV> Addressable<A> for Memory<A, V, DV>
where
    A: NumCast,
{
    fn in_range(&self, address: A) -> bool {
        match address.to_u16() {
            Some(addr) => (0..=u16::MAX).contains(&addr),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::consoles::gameboy::instruction::Instruction;
    use crate::consoles::gameboy::opcode::OpCode::EndOfProgram;
    use crate::consoles::memory::Memory;
    use crate::consoles::memory::{ECHO_OF_INTERNAL_RAM, RAM};
    use crate::consoles::readable::Readable;
    use crate::consoles::writeable::Writeable;
    use crate::utils::conversion::u16_to_u8;

    #[test]
    fn test_memory() {
        let get_default_value = || Instruction::byte_from_opcode(EndOfProgram).unwrap();
        let mut mem = Memory::<u16, u8, u16>::new(u16_to_u8, Some(Box::new(get_default_value)));
        let _ = mem.write(RAM, 100);
        assert_eq!(mem.read(RAM).unwrap(), 100);
        assert_eq!(mem.read(ECHO_OF_INTERNAL_RAM).unwrap(), 100);
        let _ = mem.write(RAM + 40, 10);
        assert_eq!(mem.read(RAM + 40).unwrap(), 10);
        assert_eq!(mem.read(ECHO_OF_INTERNAL_RAM + 40).unwrap(), 10);
    }
}
