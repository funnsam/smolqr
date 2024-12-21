fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("ec_tables.rs");

    let mut src = String::new();
    let mut last = 1;

    let mut antilog = [0_u8; 256];
    let mut log = [0_u8; 256];
    antilog[0] = 1;

    // ------------------- antilog LUT generation -------------------
    src.push_str("static ANTILOG:[u8;256]=[1,");
    for p in 1..=255 {
        log[last] = p - 1;

        let mut this = last * 2;
        if this >= 256 {
            this ^= 285;
        }
        last = this;
        src.push_str(&this.to_string());
        src.push(',');
        antilog[p as usize] = this as u8;
    }
    src.push_str("];");

    // ------------------- log LUT generation -------------------
    src.push_str("static LOG:[u8;256]=[");
    for i in log.iter() {
        src.push_str(&i.to_string());
        src.push(',');
    }
    src.push_str("];");

    // ------------------- generator polynomials coefficients LUT generation -------------------
    // NOTE: all coefficients are represented in α-notation
    src.push_str("static GEN_COEFF:[&'static[u8];256]=[&[],&[0,0],");
    let mut last = vec![0, 0_u8];
    for n in 2..=255 {
        let mut this = vec![0_u8; n as usize + 1];

        let j = last[0] as usize + n as usize - 1;
        this[0] = (j % 256 + j / 256) as u8;

        for (i, e) in this.iter_mut().enumerate().skip(1).take(n as usize - 1) {
            let j = last[i] as usize + n as usize - 1;

            *e = log[(antilog[last[i - 1] as usize] ^ antilog[j % 256 + j / 256]) as usize];
        }

        src.push_str("&[");
        for i in this.iter() {
            src.push_str(&i.to_string());
            src.push(',');
        }
        src.push_str("],");

        last = this;
    }
    src.push_str("];");

    std::fs::write(&dest_path, src).unwrap();
    println!("cargo::rerun-if-changed=build.rs");
}
