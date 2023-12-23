use std::ops::Deref;

use super::{gameboy::{game_boy::GameBoy, gbcartridge::GbCartridge}, cartridge::Cartridge};


pub trait Console {
    
    fn save_game(&self, path: String);
    fn load_save(&self, path: String);
    fn run(&mut self);

}


pub struct NoConsolePresentError {
    
    pub what: String,
}


pub fn create_for(cart: impl Cartridge) -> Result<impl Console, NoConsolePresentError> {

    match cart.as_any().downcast_ref::<GbCartridge>() {
        Some(c) => {
            return Ok(
                    GameBoy::new(c)
            );
        },
        _ => {}
    }



    Err(
        NoConsolePresentError {
            what: "No Console present for the rom provided".to_string()
        }
    )
}
