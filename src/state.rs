use camino::Utf8PathBuf;
use clap::Parser;

use crate::brotli_quality::BrotliQuality;

pub trait State {}

#[derive(Debug, Parser)]
pub struct Uninitialized {
    /// Path to the input TTF file
    #[clap()]
    pub input: Utf8PathBuf,

    /// Path to the output WOFF2 file. Defaults to the name of the input file with a .woff2
    /// extension
    #[clap(short, long)]
    pub output: Option<Utf8PathBuf>,

    /// Brotli quality, between 0 and 11 inclusive
    #[clap(short, long, default_value = "11")]
    pub quality: BrotliQuality,
}

impl State for Uninitialized {}

#[derive(Debug)]
pub struct Loaded {
    /// Font data
    pub data: Vec<u8>,

    /// Path to the input TTF file
    pub input: Utf8PathBuf,

    /// Path to the output WOFF2 file
    pub output: Utf8PathBuf,

    /// Brotli quality, between 0 and 11 inclusive
    pub quality: BrotliQuality,
}

impl State for Loaded {}
