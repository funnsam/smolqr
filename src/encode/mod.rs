pub(crate) mod alphanumeric_table;

use crate::*;

struct BitsWriter {
    pub bits: Vec<u8>,
    pub last_length: u8,
}

impl BitsWriter {
    pub fn new() -> Self {
        Self { bits: vec![0], last_length: 0 }
    }

    pub fn len(&self) -> usize {
        self.bits.len() * 8 - 8 + self.last_length as usize
    }

    pub fn write_bit(&mut self, b: bool) {
        *self.bits.last_mut().unwrap() |= (b as u8) << (7 - self.last_length);
        self.last_length += 1;

        if self.last_length >= 8 {
            self.bits.push(0);
            self.last_length = 0;
        }
    }

    pub fn write_bits(&mut self, l: usize, u: usize) {
        for i in 0..l {
            self.write_bit(u & (1 << (l - i - 1)) != 0);
        }
    }

    pub fn write_u8_aligned(&mut self, u: u8) {
        assert_eq!(self.last_length, 0);
        *self.bits.last_mut().unwrap() = u;
        self.bits.push(0);
    }

    #[allow(unused)]
    pub fn dump(&self) {
        for b in self.bits.iter() {
            print!("{b:08b} ");
        }
        println!();
    }

    pub fn align(&mut self) {
        if self.last_length != 0 {
            self.bits.push(0);
            self.last_length = 0;
        }
    }
}

pub fn encode(string: &[u8], mode: Mode, version: Version, ec: ErrorCorrectLv) -> Option<Vec<u8>> {
    let mut buffer = BitsWriter::new();
    buffer.write_bits(4, mode.indicator() as usize);
    buffer.write_bits(version.char_count_length(mode), string.len());

    match mode {
        Mode::Numeric => encode_numeric(string, &mut buffer),
        Mode::Alphanumeric => encode_alphanumeric(string, &mut buffer),
        Mode::Bytes => encode_bytes(string, &mut buffer),
        Mode::Kanji => todo!(),
    }?;

    let bytes = version.max_data_bytes(ec);

    buffer.write_bits((bytes * 8 - buffer.len()).min(4), 0);
    buffer.align();

    let mut even = true;
    for _ in buffer.bits.len()..=bytes {
        buffer.write_u8_aligned(if even { 0b11101100 } else { 0b00010001 });
        even ^= true;
    }

    buffer.bits.pop();
    Some(buffer.bits)
}

#[test]
fn test_encode() {
    assert_eq!(
        encode(b"HELLO WORLD", Mode::Alphanumeric, Version::new(1), ErrorCorrectLv::Q),
        Some(vec![
            0b00100000, 0b01011011, 0b00001011, 0b01111000,
            0b11010001, 0b01110010, 0b11011100, 0b01001101,
            0b01000011, 0b01000000, 0b11101100, 0b00010001,
            0b11101100
        ])
    );
}

fn encode_numeric(string: &[u8], buffer: &mut BitsWriter) -> Option<()> {
    for p in string.chunks(3) {
        let mut parse = 0;
        for c in p.iter() {
            parse = parse * 10 + (*c as char).to_digit(10)? as usize;
        }

        match p.len() {
            3 => buffer.write_bits(10, parse),
            2 => buffer.write_bits(7, parse),
            1 => buffer.write_bits(4, parse),
            _ => unreachable!()
        }
    }

    Some(())
}

#[test]
fn test_numeric() {
    let mut w = BitsWriter::new();
    assert!(encode_numeric(b"8675309", &mut w).is_some());

    assert_eq!(&w.bits, &[0b1101_1000, 0b1110_0001, 0b0010_1001, 0]);
    assert_eq!(w.last_length, 0);
}

fn encode_alphanumeric(string: &[u8], buffer: &mut BitsWriter) -> Option<()> {
    use alphanumeric_table::get;

    for p in string.chunks(2) {
        if p.len() == 2 {
            buffer.write_bits(11, get(p[0])? as usize * 45 + get(p[1])? as usize);
        } else {
            buffer.write_bits(6, get(p[0])? as usize);
        }
    }

    Some(())
}

#[test]
fn test_alphanumeric() {
    let mut w = BitsWriter::new();
    assert!(encode_alphanumeric(b"HELLO WORLD", &mut w).is_some());

    assert_eq!(&w.bits, &[0b0110_0001, 0b0110_1111, 0b0001_1010, 0b0010_1110, 0b0101_1011, 0b1000_1001, 0b1010_1000, 0b0110_1000]);
    assert_eq!(w.last_length, 5);
}

fn encode_bytes(string: &[u8], buffer: &mut BitsWriter) -> Option<()> {
    for b in string.iter() {
        buffer.write_bits(8, *b as usize);
    }

    Some(())
}
