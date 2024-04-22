use std::path::PathBuf;

use camino::Utf8PathBuf;
use clap::Parser;

use crate::brotli_quality::BrotliQuality;

pub trait State {}
impl State for Uninitialized {}
impl State for Loaded {}

/// A state for a converter that represents an uninitialized state.
#[derive(Debug, Parser)]
pub struct Uninitialized {
    /// Path to the input TTF file. The file name must end with the `.ttf` extension, with
    /// case-insensitive.
    #[clap()]
    pub input: Utf8PathBuf,

    /// Path to the output WOFF2 file. [`None`] means that the output file will default to the name
    /// of the input file with a `.woff2` extension.
    #[clap(short, long)]
    pub output: Option<Utf8PathBuf>,

    /// Brotli quality, between 0 and 11 inclusive
    #[clap(short, long, default_value = "11")]
    pub quality: BrotliQuality,
}

impl Default for Uninitialized {
    fn default() -> Self {
        Self::new()
    }
}

impl Uninitialized {
    pub fn new() -> Self {
        Self::parse()
    }
}

/// A state for a converter that represents a loaded state.
#[derive(Debug)]
pub struct Loaded {
    /// Font data
    pub data: Vec<u8>,

    /// Path to the output WOFF2 file. Defaults to the name of the input file with a .woff2
    /// extension
    pub output: Option<PathBuf>,

    /// Brotli quality, between 0 and 11 inclusive
    pub quality: BrotliQuality,
}
