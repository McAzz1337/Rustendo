#![allow(dead_code)]

mod gameboy {
    use std::ops::RangeInclusive;

    pub const ROM_BANK_00: RangeInclusive<usize> = 0x0000..=0x3FFF;
    pub const ROM_BANK_1_N: RangeInclusive<usize> = 0x4000..=0x7FFF;
    pub const VRAM: RangeInclusive<usize> = 0x8000..=0x9FFF;
    pub const EXTERNAL_RAM: RangeInclusive<usize> = 0xA000..=0xBFFF;
    pub const WRAM: RangeInclusive<usize> = 0xC000..=0xCFFF;
    pub const EXTERNAL_WRAM: RangeInclusive<usize> = 0xD000..=0xDFFF;
    pub const ECHO_RAM: RangeInclusive<usize> = 0xE000..=0xFDFF; // Nintendo says not to use this
    pub const OBJECT_ATTRIBUTE_MEMORY: RangeInclusive<usize> = 0xFE00..=0xFE9F;
    pub const _UNUSABLE: RangeInclusive<usize> = 0xFEA0..=0xFEFF; // Nintendo says not to use this
    pub const IO_REGISTERS: RangeInclusive<usize> = 0xFF00..=0xFF7F;
    pub const H_RAM: RangeInclusive<usize> = 0xFF80..=0xFFFE;
    pub const INTERRUPT_ENABLE_REGISTER: usize = 0xFFFF;
}
