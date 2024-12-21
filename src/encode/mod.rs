mod alphanumeric_table;

use crate::*;

fn encode(string: &str, buffer: &mut BitsWriter, mode: Mode, version: Version, ec: ErrorCorrectLv) -> Option<()> {
    buffer.write_bits(4, mode.indicator() as usize);
    buffer.write_bits(version.char_count_length(mode), string.len());

    match mode {
        Mode::Numeric => encode_numeric(string, buffer),
        Mode::Alphanumeric => encode_alphanumeric(string, buffer),
        Mode::Bytes => encode_bytes(string, buffer),
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

    Some(())
}

#[test]
fn test_encode() {
    let mut w = BitsWriter::new();
    assert!(encode("HELLO WORLD", &mut w, Mode::Alphanumeric, Version::new(1), ErrorCorrectLv::Q).is_some());

    assert_eq!(&w.bits, &[0b00100000, 0b01011011, 0b00001011, 0b01111000, 0b11010001, 0b01110010, 0b11011100, 0b01001101, 0b01000011, 0b01000000, 0b11101100, 0b00010001, 0b11101100, 0]);
    assert_eq!(w.last_length, 0);
}

fn encode_numeric(string: &str, buffer: &mut BitsWriter) -> Option<()> {
    let mut iter = string.chars().array_chunks::<3>();
    for p in &mut iter {
        let mut parse = 0;
        for c in p.iter() {
            parse = parse * 10 + c.to_digit(10)? as usize;
        }

        if p[0] != '0' {
            buffer.write_bits(10, parse);
        } else if p[1] != '0' {
            buffer.write_bits(7, parse);
        } else {
            buffer.write_bits(4, parse);
        }
    }

    if let Some(p) = iter.into_remainder() {
        let p = p.as_slice();

        let mut parse = 0;
        for c in p.iter() {
            parse = parse * 10 + c.to_digit(10)? as usize;
        }

        if p.len() == 2 {
            buffer.write_bits(7, parse);
        } else {
            buffer.write_bits(4, parse);
        }
    }

    Some(())
}

#[test]
fn test_numeric() {
    let mut w = BitsWriter::new();
    assert!(encode_numeric("8675309", &mut w).is_some());

    assert_eq!(&w.bits, &[0b1101_1000, 0b1110_0001, 0b0010_1001, 0]);
    assert_eq!(w.last_length, 0);
}

fn encode_alphanumeric(string: &str, buffer: &mut BitsWriter) -> Option<()> {
    use alphanumeric_table::LUT;

    let mut iter = string.chars().array_chunks::<2>();
    for p in &mut iter {
        buffer.write_bits(11, *LUT.get(&p[0])? as usize * 45 + *LUT.get(&p[1])? as usize);
    }

    if let Some(p) = iter.into_remainder() {
        buffer.write_bits(6, *LUT.get(&p.as_slice()[0])? as usize);
    }

    Some(())
}

#[test]
fn test_alphanumeric() {
    let mut w = BitsWriter::new();
    assert!(encode_alphanumeric("HELLO WORLD", &mut w).is_some());

    assert_eq!(&w.bits, &[0b0110_0001, 0b0110_1111, 0b0001_1010, 0b0010_1110, 0b0101_1011, 0b1000_1001, 0b1010_1000, 0b0110_1000]);
    assert_eq!(w.last_length, 5);
}

fn encode_bytes(string: &str, buffer: &mut BitsWriter) -> Option<()> {
    for b in string.as_bytes().iter() {
        buffer.write_bits(8, *b as usize);
    }

    Some(())
}
