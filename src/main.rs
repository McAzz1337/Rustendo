#[macro_use]
extern crate lazy_static;

pub mod consoles;
pub mod macros;
pub mod utils;

use consoles::cartridge::{self};
use consoles::console::{self, Console};

fn main() {
    let file = "roms/Pokemon-Silver.gbc";

    match cartridge::create(file) {
        Ok(cartridge) => match console::create_for(cartridge) {
            Ok(mut console) => {
                trace!(console.run());
            }
            Err(e) => {
                println!("{}", e);
            }
        },
        Err(e) => {
            println!("{}", e);
        }
    }
}
