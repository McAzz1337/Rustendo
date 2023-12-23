use super::gameboy::gbcartridge::GbCartridge;
use std::any::Any;

pub trait Cartridge {
    fn load(&mut self, path: String) -> Result<(), CartridgeNotFoundError>;
    fn dump(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}

pub struct CartridgeNotFoundError {

    pub what: String,
}

pub fn create(path: String) -> Result<impl Cartridge, CartridgeNotFoundError> {

    if let Some(i) = path.rfind(".") {

        let suffix: String = path.chars().skip(i + 1).take(path.len() - i - 1).collect();

        match suffix.as_str() {
            "gbc" => {
                Ok(
                    GbCartridge::new(path)
                )
            },
            _ => {
                Err(
                    CartridgeNotFoundError {
                        what: "Suffix unknown: ".to_string() + suffix.as_str(),
                    }
                )
            }
        }
    }
    else {

        Err(
            CartridgeNotFoundError {
                what: "Invalid path: ".to_string() + path.as_str(),
            }
        )
    }

}