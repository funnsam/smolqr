#![feature(array_chunks)]
#![feature(iter_array_chunks)]

pub mod encode;

struct BitsWriter {
    pub bits: Vec<u8>,
    pub last_length: u8,
}

impl BitsWriter {
    pub fn new() -> Self {
        Self { bits: vec![0], last_length: 0 }
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

    pub fn dump(&self) {
        for b in self.bits.iter() {
            print!("{b:08b} ");
        }
        println!();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorCorrectLv {
    L, M, Q, H
}

