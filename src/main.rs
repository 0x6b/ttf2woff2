use std::{
    fs::{read, write},
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser;
use ttf2woff2::{BrotliQuality, encode};

#[derive(Parser)]
#[command(about, version)]
struct Args {
    /// Path to the input TTF file
    input: PathBuf,

    /// Path to the output WOFF2 file (defaults to input with .woff2 extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Brotli compression quality (0-11)
    #[arg(short, long, default_value = "9")]
    quality: u8,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let output = args.output.unwrap_or_else(|| args.input.with_extension("woff2"));
    let quality = BrotliQuality::from(args.quality);

    let ttf_data = match read(&args.input) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading {}: {e}", args.input.display());
            return ExitCode::FAILURE;
        }
    };

    let woff2_data = match encode(&ttf_data, quality) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error encoding: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = write(&output, &woff2_data) {
        eprintln!("Error writing {}: {e}", output.display());
        return ExitCode::FAILURE;
    }

    let compression = (1.0 - woff2_data.len() as f64 / ttf_data.len() as f64) * 100.0;
    println!(
        "{} -> {} ({} -> {} bytes, {compression:.1}% compression)",
        args.input.display(),
        output.display(),
        ttf_data.len(),
        woff2_data.len()
    );

    ExitCode::SUCCESS
}
