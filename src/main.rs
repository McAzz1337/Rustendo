#[macro_use]
extern crate lazy_static;

pub mod consoles;
pub mod macros;
pub mod utils;

use consoles::cartridge::create_catridge;
use consoles::console::{create_console_for, Console};

fn main() {
    let file = "roms/Pokemon-Silver.gbc";

    match create_catridge(file) {
        Ok(cartridge) => match create_console_for(cartridge) {
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
