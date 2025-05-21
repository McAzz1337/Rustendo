use std::fmt::Display;

use super::{
    cartridge::Cartridge,
    gameboy::{game_boy::GameBoy, gbcartridge::GbCartridge},
};
use std::error::Error;

pub trait Console {
    fn save_game(&self, path: String);
    fn load_save(&self, path: String);
    fn run(&mut self);
}

#[derive(Debug)]
pub struct NoConsolePresentError {
    pub what: String,
}

impl Display for NoConsolePresentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.what)
    }
}

impl Error for NoConsolePresentError {}

pub fn create_for(cart: impl Cartridge) -> Result<impl Console, Box<dyn Error>> {
    if let Some(cartridge) = cart.as_any().downcast_ref::<GbCartridge>() {
        Ok(GameBoy::new(cartridge.clone()))
    } else {
        Err(Box::new(NoConsolePresentError {
            what: String::from("No Console present for the rom provided"),
        }))
    }
}
