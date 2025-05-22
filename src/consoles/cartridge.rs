use super::gameboy::gbcartridge::GbCartridge;
use std::error::Error;
use std::{any::Any, fmt::Display};

pub trait Cartridge {
    fn dump(&self) -> String;
    fn dump_raw(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct CartridgeNotFoundError {
    pub what: String,
}

impl Display for CartridgeNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.what)
    }
}

impl Error for CartridgeNotFoundError {}

pub fn create_catridge(path: &str) -> Result<impl Cartridge, Box<dyn Error>> {
    if let Some(i) = path.rfind(".") {
        let suffix = &path[i + 1..];

        match suffix {
            "gbc" => GbCartridge::new(path),
            _ => Err(Box::new(CartridgeNotFoundError {
                what: "Suffix unknown: ".to_string() + suffix,
            })),
        }
    } else {
        Err(Box::new(CartridgeNotFoundError {
            what: "Invalid path: ".to_string() + path,
        }))
    }
}
