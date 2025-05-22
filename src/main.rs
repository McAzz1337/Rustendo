#[macro_use]
extern crate lazy_static;

pub mod consoles;
pub mod macros;
pub mod utils;

use consoles::cartridge::{self};
use consoles::console::{self, Console};

macro_rules! _assert_eq {
    ($a: expr, $b: expr) => {
        if ($a != $b) {
            eprintln!(
                "Unexpected value [{}]\n\texpected: {}\tactual: {}",
                stringify!($a),
                $b,
                $a
            );
        }
    };
}

macro_rules! _assert {
    ($a: expr, $b: expr, $c: expr) => {
        if (!$a) {
            eprintln!("expected: {}\tactual: {}", $b, $c);
        }
        assert!($a);
    };
}

fn main() {
    let file = String::from("roms/Pokemon-Silver.gbc");

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
