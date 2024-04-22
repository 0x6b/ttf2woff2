/// Error type for the library
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to convert TTF to WOFF2 for some reason.
    #[error("Failed to convert TTF to WOFF2")]
    ConversionFailed,

    /// Failed to parse integer.
    #[error("Failed to parse integer")]
    ParseInt(#[from] std::num::ParseIntError),

    /// File name does not end with `.ttf`.
    #[error("Invalid file name: {0}. It must end with .ttf")]
    InvalidFileName(String),

    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// The output file is not specified for write operation.
    #[error("Output file is not specified")]
    OutputNotSpecified,

    /// Transparent error for I/O operations.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Transparent error for any other error.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
