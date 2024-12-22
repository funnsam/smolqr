use crate::{ErrorCorrectLv, Mode, Version};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrMatrix {
    bitmap: Vec<u8>,
    size: usize,
    stride: usize
}

impl QrMatrix {
    fn new(size: usize) -> Self {
        Self {
            bitmap: vec![0; ((size + 7) / 8) * size],
            size,
            stride: (size + 7) / 8,
        }
    }

    pub fn generate(string: &[u8], mode: Mode, version: Version, ec: ErrorCorrectLv) -> Self {
        let (mut mat, functions) = generate_unmasked_matrix(
            version,
            &crate::structure::structure(
                &crate::encode::encode(string, mode, version, ec).unwrap(),
                version,
                ec
            ),
        );
        let mask = apply_best_mask(&mut mat, &functions);
        place_format_and_version(&mut mat, version, ec, mask);

        mat
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        let b = self.bitmap[x / 8 + y * self.stride];
        b & (1 << (x % 8)) != 0
    }

    fn set(&mut self, x: usize, y: usize, v: bool) {
        self.bitmap[x / 8 + y * self.stride] &= !(1 << x % 8);
        self.bitmap[x / 8 + y * self.stride] |= (v as u8) << x % 8;
    }

    pub fn size(&self) -> usize { self.size }
}

impl core::fmt::Display for QrMatrix {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\x1b[38;5;255m")?;
        for _ in 0..4 {
            writeln!(f, "{:█<1$}", "", self.size() * 2 + 16)?;
        }

        for y in 0..self.size() {
            write!(f, "\x1b[38;5;255m████████")?;
            for b in 0..self.stride {
                let mut x = self.bitmap[y * self.stride + b];

                for i in 0..8 {
                    if b * 8 + i >= self.size() { break };

                    write!(f, "\x1b[38;5;{}m██", if x & 1 != 0 { "232" } else { "255" })?;
                    x >>= 1;
                }
            }
            writeln!(f, "\x1b[38;5;255m████████")?;
        }

        write!(f, "\x1b[38;5;255m")?;
        for _ in 0..4 {
            writeln!(f, "{:█<1$}", "", self.size() * 2 + 16)?;
        }

        write!(f, "\x1b[0m")
    }
}

struct UnfinishedMatrix {
    matrix: QrMatrix,
    done: QrMatrix,
}

impl UnfinishedMatrix {
    fn new(size: usize) -> Self {
        Self {
            matrix: QrMatrix::new(size),
            done: QrMatrix::new(size),
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<bool> {
        self.done.get(x, y).then(|| self.matrix.get(x, y))
    }

    fn set(&mut self, x: usize, y: usize, v: bool) {
        if !self.done.get(x, y) {
            self.done.set(x, y, true);
            self.matrix.set(x, y, v);
        }
    }

    fn set_hline(&mut self, x: usize, y: usize, dist: usize, v: bool) {
        for x in x..x + dist {
            self.set(x, y, v);
        }
    }

    fn set_vline(&mut self, x: usize, y: usize, dist: usize, v: bool) {
        for y in y..y + dist {
            self.set(x, y, v);
        }
    }

    fn set_filled_box(&mut self, x: usize, y: usize, sx: usize, sy: usize, v: bool) {
        for y in y..y + sy {
            for x in x..x + sx {
                self.set(x, y, v);
            }
        }
    }

    fn set_outline_box(&mut self, x: usize, y: usize, sx: usize, sy: usize, v: bool) {
        for x in x..x + sx {
            self.set(x, y, v);
            self.set(x, y + sy - 1, v);
        }
        for y in y..y + sy {
            self.set(x, y, v);
            self.set(x + sx - 1, y, v);
        }
    }
}

fn generate_unmasked_matrix(version: Version, data: &[u8]) -> (QrMatrix, QrMatrix) {
    let size = version.0 as usize * 4 + 21;
    let mut mat = UnfinishedMatrix::new(size);

    place_finder(&mut mat, 0, 0);
    place_finder(&mut mat, size - 7, 0);
    place_finder(&mut mat, 0, size - 7);

    for y in ALIGN_LOCATIONS[version.0 as usize] {
        for x in ALIGN_LOCATIONS[version.0 as usize] {
            place_alignment(&mut mat, *x, *y);
        }
    }

    // timing
    for i in 8..size - 8 {
        mat.set(i, 6, i & 1 == 0);
        mat.set(6, i, i & 1 == 0);
    }

    // dark module
    mat.set(8, 4 * version.version() as usize + 9, true);

    // reserved area
    mat.set_hline(0, 8, 9, false);
    mat.set_vline(8, 0, 9, false);
    mat.set_hline(size - 8, 8, 8, false);
    mat.set_vline(8, size - 7, 7, false);

    if version.version() >= 7 {
        mat.set_filled_box(size - 11, 0, 3, 6, false);
        mat.set_filled_box(0, size - 11, 6, 3, false);
    }

    let functions = mat.done.clone();

    // data placement
    let mut cursor = (size - 1, size - 1, true, true);
    for mut b in data.iter().copied() {
        for _ in 0..8 {
            place_data(&mut mat, &mut cursor, b & 0x80 != 0);
            b <<= 1;
        }
    }

    (mat.matrix, functions)
}

fn place_finder(mat: &mut UnfinishedMatrix, x: usize, y: usize) {
    mat.set_outline_box(x, y, 7, 7, true);
    mat.set_outline_box(x + 1, y + 1, 5, 5, false);
    mat.set_filled_box(x + 2, y + 2, 3, 3, true);

    mat.set_vline(if x == 0 { 7 } else { x - 1 }, y, 7, false);
    mat.set_hline(x, if y == 0 { 7 } else { y - 1 }, 7, false);

    mat.set(if x == 0 { 7 } else { x - 1 }, if y == 0 { 7 } else { y - 1 }, false);
}

fn place_alignment(mat: &mut UnfinishedMatrix, x: usize, y: usize) {
    if mat.done.get(x, y) { return };

    mat.set_outline_box(x - 2, y - 2, 5, 5, true);
    mat.set_outline_box(x - 1, y - 1, 3, 3, false);
    mat.set(x, y, true);
}

fn place_data(mat: &mut UnfinishedMatrix, cursor: &mut (usize, usize, bool, bool), data: bool) {
    mat.set(cursor.0, cursor.1, data);

    while mat.get(cursor.0, cursor.1).is_some() {
        if cursor.2 {
            // upward
            if cursor.3 {
                // at right
                cursor.0 -= 1;
                cursor.3 = false;
            } else {
                // at left
                if cursor.1 == 0 {
                    // at edge
                    if cursor.0 == 0 { break };
                    cursor.0 -= 1 + (cursor.0 == 7) as usize;
                    cursor.2 = false;
                } else {
                    cursor.0 += 1;
                    cursor.1 -= 1;
                    cursor.3 = true;
                }
            }
        } else {
            // downward
            if cursor.3 {
                // at left
                if cursor.1 >= mat.matrix.size() - 1 {
                    // at edge
                    if cursor.0 == 0 { break };
                    cursor.0 -= 1 + (cursor.0 == 7) as usize;
                    cursor.2 = true;
                } else {
                    cursor.0 += 1;
                    cursor.1 += 1;
                    cursor.3 = false;
                }
            } else {
                // at right
                cursor.0 -= 1;
                cursor.3 = true;
            }
        }
    }
}

fn apply_best_mask(mat: &mut QrMatrix, functions: &QrMatrix) -> usize {
    let mut try_mat = mat.clone();
    let mut best_mask = 0;
    let mut best_penalty = usize::MAX;

    for m in 0..8 {
        apply_mask(&mut try_mat, functions, m);
        let penalty = calculate_penalty(&try_mat);

        if best_penalty > penalty {
            best_mask = m;
            best_penalty = penalty;
        }

        try_mat.bitmap.copy_from_slice(&mat.bitmap);
    }

    apply_mask(mat, functions, best_mask);
    best_mask
}

fn calculate_penalty(mat: &QrMatrix) -> usize {
    let mut penalty = 0;
    let mut black_count = 0;

    // 5 in a row
    let mut count = 0;
    let mut color = false;
    for y in 0..mat.size() {
        for x in 0..mat.size() {
            let c = mat.get(x, y);
            black_count += c as usize;
            count += (color == c) as usize;
            color = c;

            if count == 5 {
                penalty += 3;
            } else if count > 5 {
                penalty += 1;
            }
        }
    }

    // 5 in a column
    count = 0;
    color = false;
    for x in 0..mat.size() {
        for y in 0..mat.size() {
            let c = mat.get(x, y);
            count += (color == c) as usize;
            color = c;

            if count == 5 {
                penalty += 3;
            } else if count > 5 {
                penalty += 1;
            }
        }
    }

    // 2x2 overlapping squares
    for y in 0..mat.size() - 1 {
        for x in 0..mat.size() - 1 {
            let tl = mat.get(x, y);
            let tr = mat.get(x + 1, y);
            let bl = mat.get(x, y + 1);
            let br = mat.get(x + 1, y + 1);

            if tl == tr && tr == bl && bl == br {
                penalty += 3;
            }
        }
    }

    // finder 1-1-3-1-1 patterns in rows
    for y in 0..mat.size() {
        for x in 0..mat.size() - 6 {
            let a = mat.get(x, y);
            let b = mat.get(x + 1, y);
            let c = mat.get(x + 2, y);
            let d = mat.get(x + 3, y);
            let e = mat.get(x + 4, y);
            let f = mat.get(x + 5, y);
            let g = mat.get(x + 6, y);

            if a && !b && c && d && e && !f && g {
                if (
                    x >= 4
                    && !mat.get(x - 1, y)
                    && !mat.get(x - 2, y)
                    && !mat.get(x - 3, y)
                    && !mat.get(x - 4, y)
                ) || (
                    x + 7 < mat.size() - 4
                    && !mat.get(x + 7, y)
                    && !mat.get(x + 8, y)
                    && !mat.get(x + 9, y)
                    && !mat.get(x + 10, y)
                ) {
                    penalty += 40;
                }
            }
        }
    }

    // finder 1-1-3-1-1 patterns in columns
    for x in 0..mat.size() {
        for y in 0..mat.size() - 6 {
            let a = mat.get(x, y);
            let b = mat.get(x, y + 1);
            let c = mat.get(x, y + 2);
            let d = mat.get(x, y + 3);
            let e = mat.get(x, y + 4);
            let f = mat.get(x, y + 5);
            let g = mat.get(x, y + 6);

            if a && !b && c && d && e && !f && g {
                if (
                    y >= 4
                    && !mat.get(x, y - 1)
                    && !mat.get(x, y - 2)
                    && !mat.get(x, y - 3)
                    && !mat.get(x, y - 4)
                ) || (
                    y + 7 < mat.size() - 4
                    && !mat.get(x, y + 7)
                    && !mat.get(x, y + 8)
                    && !mat.get(x, y + 9)
                    && !mat.get(x, y + 10)
                ) {
                    penalty += 40;
                }
            }
        }
    }

    // black ratio
    let percentage = black_count * 100 / (mat.size() * mat.size());
    let prev = (percentage / 5 * 5) as isize;
    penalty += (prev - 50).abs().min((prev - 45).abs()) as usize * 10;

    penalty
}

fn apply_mask(mat: &mut QrMatrix, functions: &QrMatrix, mask: usize) {
    for y in 0..mat.size() {
        for x in 0..mat.size() {
            if !functions.get(x, y) && match mask {
                0 => (x + y) % 2,
                1 => y % 2,
                2 => x % 3,
                3 => (x + y) % 3,
                4 => (y / 2 + x / 3) % 2,
                5 => x * y % 2 + x * y % 3,
                6 => (x * y % 2 + x * y % 3) % 2,
                7 => ((x + y) % 2 + x * y % 3) % 2,
                _ => panic!(),
            } == 0 {
                mat.set(x, y, !mat.get(x, y));
            }
        }
    }
}

fn place_format_and_version(mat: &mut QrMatrix, version: Version, ec: ErrorCorrectLv, mask: usize) {
    let format = FORMAT_INFO[ec as usize * 8 + mask];

    for i in 0..=5 {
        mat.set(i, 8, (format << i) & 0x4000 != 0);
        mat.set(8, mat.size() - i - 1, (format << i) & 0x4000 != 0);
    }
    mat.set(7, 8, (format << 6) & 0x4000 != 0);
    mat.set(8, mat.size() - 6 - 1, (format << 6) & 0x4000 != 0);

    for i in 7..=8 {
        mat.set(8, 15 - i, (format << i) & 0x4000 != 0);
        mat.set(mat.size() - 15 + i, 8, (format << i) & 0x4000 != 0);
    }

    for i in 9..=14 {
        mat.set(8, 14 - i, (format << i) & 0x4000 != 0);
        mat.set(mat.size() - 15 + i, 8, (format << i) & 0x4000 != 0);
    }

    if version.version() < 7 { return };

    let mut version = VERSION_INFO[version.0 as usize];

    for i in 0..6 {
        for j in 0..3 {
            let v = version & 1 != 0;

            mat.set(mat.size() + j - 11, i, v);
            mat.set(i, mat.size() + j - 11, v);

            version >>= 1;
        }
    }
}

static ALIGN_LOCATIONS: [&[usize]; 40] = [
    &[],
    &[6, 18],
    &[6, 22],
    &[6, 26],
    &[6, 30],
    &[6, 34],
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
];

// https://www.thonky.com/qr-code-tutorial/format-version-tables
static FORMAT_INFO: [u16; 4 * 8] = [
    0b111011111000100,
    0b111001011110011,
    0b111110110101010,
    0b111100010011101,
    0b110011000101111,
    0b110001100011000,
    0b110110001000001,
    0b110100101110110,
    0b101010000010010,
    0b101000100100101,
    0b101111001111100,
    0b101101101001011,
    0b100010111111001,
    0b100000011001110,
    0b100111110010111,
    0b100101010100000,
    0b011010101011111,
    0b011000001101000,
    0b011111100110001,
    0b011101000000110,
    0b010010010110100,
    0b010000110000011,
    0b010111011011010,
    0b010101111101101,
    0b001011010001001,
    0b001001110111110,
    0b001110011100111,
    0b001100111010000,
    0b000011101100010,
    0b000001001010101,
    0b000110100001100,
    0b000100000111011,
];

static VERSION_INFO: [u32; 40] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0b000111110010010100,
    0b001000010110111100,
    0b001001101010011001,
    0b001010010011010011,
    0b001011101111110110,
    0b001100011101100010,
    0b001101100001000111,
    0b001110011000001101,
    0b001111100100101000,
    0b010000101101111000,
    0b010001010001011101,
    0b010010101000010111,
    0b010011010100110010,
    0b010100100110100110,
    0b010101011010000011,
    0b010110100011001001,
    0b010111011111101100,
    0b011000111011000100,
    0b011001000111100001,
    0b011010111110101011,
    0b011011000010001110,
    0b011100110000011010,
    0b011101001100111111,
    0b011110110101110101,
    0b011111001001010000,
    0b100000100111010101,
    0b100001011011110000,
    0b100010100010111010,
    0b100011011110011111,
    0b100100101100001011,
    0b100101010000101110,
    0b100110101001100100,
    0b100111010101000001,
    0b101000110001101001,
];
