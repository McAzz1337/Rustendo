use std::ops::RangeInclusive;

pub trait Addressable<A> {
    fn assign_address_range(&mut self, range: RangeInclusive<usize>);
    fn in_range(&self, address: A) -> bool;
}
