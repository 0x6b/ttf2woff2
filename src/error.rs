#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to convert TTF to WOFF2")]
    ConversionFailed,

    #[error("Failed to parse integer")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Got invalid Brotli quality: {0}")]
    InvalidBrotliQuality(u8),

    #[error("Invalid file name: {0}. It must end with .ttf")]
    InvalidFileName(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Output file is not specified")]
    OutputNotSpecified,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
