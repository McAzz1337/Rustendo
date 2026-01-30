#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate function_name;

pub mod consoles;
pub mod filio;
pub mod macros;
pub mod utils;

use std::error::Error;

use consoles::cartridge::{Cartridge, create_catridge};
use consoles::console::{Console, create_console_for};

fn init_console_and_cartridge(path: &str) -> Result<impl Console, Box<dyn Error>> {
    let cartridge = create_catridge(path)?;
    // println!("{}", cartridge.dump());
    create_console_for(cartridge)
}

fn main() {
    let path = "roms/Pokemon-Silver.gbc";

    match init_console_and_cartridge(path) {
        Ok(mut console) => {
            trace!(console.run());
        }
        Err(e) => println!("{e}"),
    }
}
