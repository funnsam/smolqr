use std::borrow::Cow;

use clap::*;
use smolqr::*;

#[derive(Parser)]
struct Args {
    string: String,
    ec: _ErrorCorrectLv,
    #[arg(short, long)]
    mode: Option<_Mode>,
    #[arg(short, long)]
    version: Option<u8>,

    #[command(subcommand)]
    output: OutputMode,
}

#[derive(Subcommand, Clone)]
enum OutputMode {
    Print,
    Gif {
        #[arg(long, short, default_value_t = 1)]
        upscale: usize,
        #[arg(long, short, default_value = "ffffff")]
        white_color: String,
        #[arg(long, short, default_value = "000000")]
        black_color: String,
        path: String,
    },
}

#[derive(ValueEnum, Clone)]
pub enum _Mode {
    Numeric,
    Alphanumeric,
    Bytes,
    // Kanji,
}

#[derive(ValueEnum, Clone)]
pub enum _ErrorCorrectLv {
    L, M, Q, H
}

impl From<_Mode> for Mode {
    fn from(value: _Mode) -> Self {
        match value {
            _Mode::Numeric => Self::Numeric,
            _Mode::Alphanumeric => Self::Alphanumeric,
            _Mode::Bytes => Self::Bytes,
        }
    }
}

impl From<_ErrorCorrectLv> for ErrorCorrectLv {
    fn from(value: _ErrorCorrectLv) -> Self {
        match value {
            _ErrorCorrectLv::L => Self::L,
            _ErrorCorrectLv::M => Self::M,
            _ErrorCorrectLv::Q => Self::Q,
            _ErrorCorrectLv::H => Self::H,
        }
    }
}

fn main() {
    let args = Args::parse();

    let ec = args.ec.into();
    let mode = args.mode.map_or_else(
        || Mode::best_mode(args.string.as_bytes()),
        |m| m.into(),
    );
    let version = args.version.map_or_else(
        || Version::smallest_version(args.string.len(), ec, mode).unwrap(),
        |m| Version::new(m),
    );

    let mat = QrMatrix::generate(args.string.as_bytes(), mode, version, ec);

    match args.output {
        OutputMode::Print => print!("{mat}"),
        OutputMode::Gif { upscale, white_color, black_color, path } => {
            use gif::*;

            let size = ((mat.size() + 8) * upscale).try_into().unwrap();

            let mut m = vec![0; size as usize * size as usize];
            for y in 0..mat.size() {
                for x in 0..mat.size() {
                    if mat.get(x, y) {
                        for sy in 0..upscale {
                            for sx in 0..upscale {
                                m[((y + 4) * upscale + sy) * size as usize + (x + 4) * upscale + sx] = 1;
                            }
                        }
                    }
                }
            }

            let black = u32::from_str_radix(&black_color, 16).expect("failed to parse color");
            let white = u32::from_str_radix(&white_color, 16).expect("failed to parse color");

            let palette = [
                (white >> 16) as u8, (white >> 8) as u8, white as u8,
                (black >> 16) as u8, (black >> 8) as u8, black as u8,
            ];

            let mut image = std::fs::File::create(path).unwrap();
            let mut encoder = Encoder::new(&mut image, size, size, &palette).unwrap();

            let mut frame = Frame::default();
            frame.width = size;
            frame.height = size;
            frame.buffer = Cow::Owned(m);
            encoder.write_frame(&frame).unwrap();
        },
    }
}
