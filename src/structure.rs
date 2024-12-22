use crate::{err_corr::generate_ec, ErrorCorrectLv, Version};

pub fn structure(data: &[u8], version: Version, ec: ErrorCorrectLv) -> Vec<u8> {
    let blocks_data = version.blocks_data(ec);
    let g2_off = blocks_data.g1_blocks * blocks_data.g1_bytes;

    let mut result = Vec::with_capacity(data.len() + blocks_data.ec_bytes * (blocks_data.g1_blocks + blocks_data.g2_blocks));

    let g1 = (0..blocks_data.g1_blocks)
        .map(|i| &data[i * blocks_data.g1_bytes..(i + 1) * blocks_data.g1_bytes])
        .collect::<Vec<_>>();
    let g2 = (0..blocks_data.g2_blocks)
        .map(|i| &data[i * blocks_data.g2_bytes + g2_off..(i + 1) * blocks_data.g2_bytes + g2_off])
        .collect::<Vec<_>>();

    for i in 0..blocks_data.g1_bytes.max(blocks_data.g2_bytes) {
        if i < blocks_data.g1_bytes {
            for b in g1.iter() {
                result.push(b[i]);
            }
        }
        if i < blocks_data.g2_bytes {
            for b in g2.iter() {
                result.push(b[i]);
            }
        }
    }

    let ec1 = g1.iter().map(|i| {
        let mut i = i.to_vec();
        i.reverse();
        generate_ec(&i, blocks_data.ec_bytes)
    }).collect::<Vec<_>>();
    let ec2 = g2.iter().map(|i| {
        let mut i = i.to_vec();
        i.reverse();
        generate_ec(&i, blocks_data.ec_bytes)
    }).collect::<Vec<_>>();

    for i in (0..blocks_data.ec_bytes).rev() {
        for b in ec1.iter() {
            result.push(b[i]);
        }
        for b in ec2.iter() {
            result.push(b[i]);
        }
    }

    result
}

#[test]
fn test() {
    let m = structure(&[
        0b01000011,
        0b01010101,
        0b01000110,
        0b10000110,
        0b01010111,
        0b00100110,
        0b01010101,
        0b11000010,
        0b01110111,
        0b00110010,
        0b00000110,
        0b00010010,
        0b00000110,
        0b01100111,
        0b00100110,
        0b11110110,
        0b11110110,
        0b01000010,
        0b00000111,
        0b01110110,
        0b10000110,
        0b11110010,
        0b00000111,
        0b00100110,
        0b01010110,
        0b00010110,
        0b11000110,
        0b11000111,
        0b10010010,
        0b00000110,
        0b10110110,
        0b11100110,
        0b11110111,
        0b01110111,
        0b00110010,
        0b00000111,
        0b01110110,
        0b10000110,
        0b01010111,
        0b00100110,
        0b01010010,
        0b00000110,
        0b10000110,
        0b10010111,
        0b00110010,
        0b00000111,
        0b01000110,
        0b11110111,
        0b01110110,
        0b01010110,
        0b11000010,
        0b00000110,
        0b10010111,
        0b00110010,
        0b00010000,
        0b11101100,
        0b00010001,
        0b11101100,
        0b00010001,
        0b11101100,
        0b00010001,
        0b11101100,
    ], Version::new(5), ErrorCorrectLv::Q);
    assert_eq!(&m, &[
        0b01000011,
        0b11110110,
        0b10110110,
        0b01000110,
        0b01010101,
        0b11110110,
        0b11100110,
        0b11110111,
        0b01000110,
        0b01000010,
        0b11110111,
        0b01110110,
        0b10000110,
        0b00000111,
        0b01110111,
        0b01010110,
        0b01010111,
        0b01110110,
        0b00110010,
        0b11000010,
        0b00100110,
        0b10000110,
        0b00000111,
        0b00000110,
        0b01010101,
        0b11110010,
        0b01110110,
        0b10010111,
        0b11000010,
        0b00000111,
        0b10000110,
        0b00110010,
        0b01110111,
        0b00100110,
        0b01010111,
        0b00010000,
        0b00110010,
        0b01010110,
        0b00100110,
        0b11101100,
        0b00000110,
        0b00010110,
        0b01010010,
        0b00010001,
        0b00010010,
        0b11000110,
        0b00000110,
        0b11101100,
        0b00000110,
        0b11000111,
        0b10000110,
        0b00010001,
        0b01100111,
        0b10010010,
        0b10010111,
        0b11101100,
        0b00100110,
        0b00000110,
        0b00110010,
        0b00010001,
        0b00000111,
        0b11101100,
        0b11010101,
        0b01010111,
        0b10010100,
        0b11101011,
        0b11000111,
        0b11001100,
        0b01110100,
        0b10011111,
        0b00001011,
        0b01100000,
        0b10110001,
        0b00000101,
        0b00101101,
        0b00111100,
        0b11010100,
        0b10101101,
        0b01110011,
        0b11001010,
        0b01001100,
        0b00011000,
        0b11110111,
        0b10110110,
        0b10000101,
        0b10010011,
        0b11110001,
        0b01111100,
        0b01001011,
        0b00111011,
        0b11011111,
        0b10011101,
        0b11110010,
        0b00100001,
        0b11100101,
        0b11001000,
        0b11101110,
        0b01101010,
        0b11111000,
        0b10000110,
        0b01001100,
        0b00101000,
        0b10011010,
        0b00011011,
        0b11000011,
        0b11111111,
        0b01110101,
        0b10000001,
        0b11100110,
        0b10101100,
        0b10011010,
        0b11010001,
        0b10111101,
        0b01010010,
        0b01101111,
        0b00010001,
        0b00001010,
        0b00000010,
        0b01010110,
        0b10100011,
        0b01101100,
        0b10000011,
        0b10100001,
        0b10100011,
        0b11110000,
        0b00100000,
        0b01101111,
        0b01111000,
        0b11000000,
        0b10110010,
        0b00100111,
        0b10000101,
        0b10001101,
        0b11101100,
    ]);
}
