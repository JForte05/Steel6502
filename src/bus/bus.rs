use crate::memory::memory::{RAMSegment, ROMSegment};

pub enum BusError{
    UnmappedAddress(u16),
    UnsupportedOperation(u16, BusOperation)
}
pub enum BusOperation{
    Read,
    Write
}

pub trait Bus{
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, val: u8);
}

pub struct Machine{
    rom: ROMSegment,
    ram: RAMSegment,

    total_pages_used: usize
}
