use std::{
    fs::{read, write},
    num::NonZeroUsize,
    path::PathBuf,
    process::ExitCode,
    thread,
};

use clap::Parser;
use ttf2woff2::{BrotliQuality, EncodeOptions, encode_with_options};

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

    /// Number of threads for Brotli compression: 1=single-threaded (deterministic), 0=all cores, N=N threads.
    ///
    /// Multi-threaded Brotli (`-t 0` or `-t >=2`) is much faster on large fonts at
    /// quality 10-11 but produces output whose bytes depend on the thread count;
    /// total size grows by typically < 0.5 %. The output is still a valid Brotli
    /// stream that any spec-compliant WOFF2 decoder accepts.
    #[arg(short, long, default_value = "1")]
    threads: usize,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let output = args.output.unwrap_or_else(|| args.input.with_extension("woff2"));
    let quality = BrotliQuality::from(args.quality);

    let threads = match args.threads {
        0 => thread::available_parallelism().ok(),
        1 => None,
        n => NonZeroUsize::new(n),
    };

    let options = EncodeOptions { quality, threads, ..EncodeOptions::default() };

    let ttf_data = match read(&args.input) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading {}: {e}", args.input.display());
            return ExitCode::FAILURE;
        }
    };

    let woff2_data = match encode_with_options(&ttf_data, options) {
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
