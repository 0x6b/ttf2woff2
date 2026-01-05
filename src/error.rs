use std::{io, num::ParseIntError};
/// Error type for the library
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to parse TTF/OTF font
    #[error("Failed to parse font: {0}")]
    ParseError(String),

    /// Failed to encode to WOFF2
    #[error("Failed to encode: {0}")]
    EncodeError(String),

    /// Failed to parse integer
    #[error("Failed to parse integer")]
    ParseInt(#[from] ParseIntError),

    /// I/O error
    #[error(transparent)]
    Io(#[from] io::Error),
}
