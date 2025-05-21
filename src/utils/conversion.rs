use std::ops::{BitAnd, Shr};

use num_traits::{FromPrimitive, NumCast, ToPrimitive};

pub fn cast<N: NumCast + ToPrimitive, U: NumCast>(n: N) -> U {
    NumCast::from(n).unwrap()
}

pub fn u16_to_u8<T, U>(n: T) -> Option<(U, U)>
where
    T: NumCast + BitAnd<u16> + Shr<i32>,
    U: NumCast,
{
    match n.to_u16() {
        Some(n) => {
            let upper = (n >> 8) as u8;
            let lower = (n & 0xFF) as u8;
            Some((NumCast::from(upper).unwrap(), NumCast::from(lower).unwrap()))
        }
        None => None,
    }
}

pub fn u8_as_bit_string(x: u8) -> String {
    let mut bits = String::from("0b");
    for i in (0..8).rev() {
        let mask = 0b1 << i;

        let bit = x & mask;
        if bit >= 1 {
            bits = bits + "1";
        } else {
            bits = bits + "0";
        }
    }

    bits
}

pub fn u16_as_bit_string(x: u16) -> String {
    let mut bits = String::from("0b");
    for i in (0..16).rev() {
        let mask = 0b1 << i;

        let bit = x & mask;
        if bit >= 1 {
            bits = bits + "1";
        } else {
            bits = bits + "0";
        }
    }

    bits
}

pub fn u8_as_hex_string(x: u8) -> String {
    let mut hex = String::from("0x");
    let upper = (x & (0b1111 << 4)) >> 4;
    let lower = x & (0b1111);

    match upper {
        0 => hex = hex + "0",
        1 => hex = hex + "1",
        2 => hex = hex + "2",
        3 => hex = hex + "3",
        4 => hex = hex + "4",
        5 => hex = hex + "5",
        6 => hex = hex + "6",
        7 => hex = hex + "7",
        8 => hex = hex + "8",
        9 => hex = hex + "9",
        10 => hex = hex + "A",
        11 => hex = hex + "B",
        12 => hex = hex + "C",
        13 => hex = hex + "D",
        14 => hex = hex + "E",
        15 => hex = hex + "F",
        _ => {}
    }

    match lower {
        0 => hex = hex + "0",
        1 => hex = hex + "1",
        2 => hex = hex + "2",
        3 => hex = hex + "3",
        4 => hex = hex + "4",
        5 => hex = hex + "5",
        6 => hex = hex + "6",
        7 => hex = hex + "7",
        8 => hex = hex + "8",
        9 => hex = hex + "9",
        10 => hex = hex + "A",
        11 => hex = hex + "B",
        12 => hex = hex + "C",
        13 => hex = hex + "D",
        14 => hex = hex + "E",
        15 => hex = hex + "F",
        _ => {}
    }

    hex
}

pub fn u16_as_hex_string(x: u16) -> String {
    let mut hex = String::from("0x");

    for i in (0..4).rev() {
        let shift = i * 4;
        let shifted = (x >> shift) & 0b1111;

        match shifted {
            0 => hex = hex + "0",
            1 => hex = hex + "1",
            2 => hex = hex + "2",
            3 => hex = hex + "3",
            4 => hex = hex + "4",
            5 => hex = hex + "5",
            6 => hex = hex + "6",
            7 => hex = hex + "7",
            8 => hex = hex + "8",
            9 => hex = hex + "9",
            10 => hex = hex + "A",
            11 => hex = hex + "B",
            12 => hex = hex + "C",
            13 => hex = hex + "D",
            14 => hex = hex + "E",
            15 => hex = hex + "F",
            _ => {}
        }
    }

    hex
}

#[test]
fn test_u8_as_bit_string() {
    assert!(u8_as_bit_string(5) == "0b00000101");
    assert!(u8_as_bit_string(127) == "0b01111111");
    assert!(u8_as_bit_string(128) == "0b10000000");
    assert!(u8_as_bit_string(255) == "0b11111111");
}

#[test]
fn test_u16_as_bit_string() {
    assert!(u16_as_bit_string(256) == "0b0000000100000000");
    assert!(u16_as_bit_string(512) == "0b0000001000000000");
}

#[test]
fn test_u8_as_hex_string() {
    assert!(u8_as_hex_string(5) == "0x05");
    assert!(u8_as_hex_string(10) == "0x0A");
    assert!(u8_as_hex_string(15) == "0x0F");
    assert!(u8_as_hex_string(16) == "0x10");
    assert!(u8_as_hex_string(255) == "0xFF");
}
