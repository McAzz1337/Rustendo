use super::super::console::Console;
use super::gbcartridge::GbCartridge;
use crate::consoles::gameboy::cpu::Cpu;
use crate::consoles::gameboy::memory::Memory;
use crate::utils::conversion::u16_to_u8;

pub struct GameBoy {
    cpu: Cpu,
    memory: Memory<u16, u8, u16>,
    cartridge: GbCartridge,
}

impl GameBoy {
    pub fn new(cartridge: &GbCartridge) -> GameBoy {
        GameBoy {
            cpu: Cpu::new(),
            memory: Memory::<u16, u8, u16>::new(u16_to_u8),
            cartridge: cartridge.clone(),
        }
    }
}

impl Console for GameBoy {
    fn save_game(&self, path: String) {}

    fn load_save(&self, path: String) {}

    fn run(&mut self) {
        println!("dumped");
        //self.cpu.run();
    }
}
