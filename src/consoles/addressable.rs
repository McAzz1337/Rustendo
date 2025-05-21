pub trait Addressable<A> {
    fn in_range(&self, address: A) -> bool;
}
