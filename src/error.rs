use std::{io, num::ParseIntError};

/// Error type for the library
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Data too short for expected structure
    #[error("Data too short: {context}")]
    DataTooShort { context: &'static str },

    /// Unsupported font format
    #[error("Unsupported font format: expected TTF (0x00010000) or OTF (OTTO)")]
    UnsupportedFormat,

    /// Table extends beyond data bounds
    #[error("Table extends beyond data")]
    TableOutOfBounds,

    /// Invalid glyph data
    #[error("Invalid glyph: {0}")]
    InvalidGlyph(&'static str),

    /// Brotli compression failed
    #[error("Brotli compression failed: {0}")]
    Compression(String),

    /// Failed to parse integer
    #[error("Failed to parse integer")]
    ParseInt(#[from] ParseIntError),

    /// I/O error
    #[error(transparent)]
    Io(#[from] io::Error),
}
