include!(concat!(env!("OUT_DIR"), "/ec_tables.rs"));

fn long_div(a: &[u8], b: &[u8], pad_a: usize) -> Vec<u8> {
    let mut result = vec![0_u8; a.len() + pad_a];
    result[pad_a..].copy_from_slice(a);

    result
}

#[test]
fn test_long_div() {
    assert_eq!(long_div(&[
            17,
            236,
            17,
            236,
            17,
            236,
            64,
            67,
            77,
            220,
            114,
            209,
            120,
            11,
            91,
            32,
    ], &[
        ANTILOG[45],
        ANTILOG[32],
        ANTILOG[94],
        ANTILOG[64],
        ANTILOG[70],
        ANTILOG[118],
        ANTILOG[61],
        ANTILOG[46],
        ANTILOG[67],
        ANTILOG[251],
        ANTILOG[0],
    ], 10), vec![
        23,
        93,
        226,
        231,
        215,
        235,
        119,
        39,
        35,
        196,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]);
}
