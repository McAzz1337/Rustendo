use std::cell::RefCell;
use std::rc::Rc;

use super::super::console::Console;
use super::gbcartridge::GbCartridge;
use super::instruction::Instruction;
use super::opcode::OpCode::EndOfProgram;
use crate::consoles::addressable::Addressable;
use crate::consoles::bus::Bus;
use crate::consoles::gameboy::cpu::Cpu;
use crate::consoles::memory::Memory;
use crate::consoles::memory_map::gameboy::{ROM_BANK_00, WRAM};
use crate::utils::conversion::u16_to_u8;

pub type GbMemory = Memory<u16, u8, u16, 0x10000>;
pub type GbBus = Bus<u16, u8, u16>;

pub struct GameBoy {
    cpu: Cpu,
}

impl GameBoy {
    pub fn new(mut cartridge: GbCartridge) -> GameBoy {
        cartridge.assign_address_range(ROM_BANK_00);

        let get_default_value = || Instruction::byte_from_opcode(EndOfProgram).unwrap();
        let memory = Rc::new(RefCell::new(GbMemory::new(
            u16_to_u8,
            Some(Box::new(get_default_value)),
        )));
        memory.borrow_mut().assign_address_range(WRAM);

        let mut bus = GbBus::new();
        bus.connect_readable(memory.clone());
        bus.connect_writeable(memory);
        bus.connect_readable(Rc::new(RefCell::new(cartridge)));
        let bus = Rc::new(RefCell::new(bus));
        GameBoy {
            cpu: Cpu::new(bus.clone()),
        }
    }
}

impl Console for GameBoy {
    fn save_game(&self, path: String) {}

    fn load_save(&self, path: String) {}

    fn run(&mut self) {
        self.cpu.run();
    }
}
