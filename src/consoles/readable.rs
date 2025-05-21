use std::error::Error;

pub trait Readable<A, V> {
    fn read(&self, address: A) -> Result<V, Box<dyn Error>>;
}
