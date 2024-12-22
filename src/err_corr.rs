include!(concat!(env!("OUT_DIR"), "/ec_tables.rs"));

pub fn generate_ec(a: &[u8], bytes: usize) -> Vec<u8> {
    let mut r = long_div(a, GEN_COEFF[bytes], bytes);
    r.drain(bytes..);
    r
}

fn long_div(a: &[u8], b: &[u8], pad_a: usize) -> Vec<u8> {
    let mut result = vec![0_u8; a.len() + pad_a];
    result[pad_a..].copy_from_slice(a);

    for _i in 0..a.len() {
        let i = result.len() - _i - 1;

        let c = LOG[result[i] as usize];

        for (a, b) in result.iter_mut().rev()
            .skip(_i)
            .zip(b.iter().rev())
        {
            let d = *b as usize + c as usize;

            *a ^= ANTILOG[d % 256 + d / 256];
        }
    }

    result
}

#[test]
fn test() {
    assert_eq!(generate_ec(&[
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
    ]);
}
