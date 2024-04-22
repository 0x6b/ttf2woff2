use camino::Utf8PathBuf;
use clap::Parser;

use crate::brotli_quality::BrotliQuality;

pub trait State {}
impl State for Uninitialized {}
impl State for Loaded {}

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
    pub output: Option<Utf8PathBuf>,

    /// Brotli quality, between 0 and 11 inclusive
    pub quality: BrotliQuality,
}

impl State for Loaded {}
