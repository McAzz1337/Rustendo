pub trait Addressable {
    fn in_range(&self, address: u16) -> bool;
}
