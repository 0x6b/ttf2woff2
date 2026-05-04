use std::str::FromStr;

use crate::Error;

/// [Brotli](https://github.com/google/brotli/) compression quality.
///
/// An integer from 0 (fastest, largest output) to 11 (slowest, smallest output).
/// Values above 11 are clamped to 11. Defaults to 11.
///
/// Construct via [`From<u8>`] or [`FromStr`], and convert into the underlying
/// `u8` or `i32` (as required by the `brotli` crate) via the corresponding
/// `From` impls.
#[derive(Debug, Clone, Copy)]
pub struct BrotliQuality {
    value: u8,
}

impl Default for BrotliQuality {
    fn default() -> Self {
        Self { value: 11 }
    }
}

impl From<u8> for BrotliQuality {
    /// Create a new BrotliQuality with the given quality.
    /// If the quality is greater than 11, it is clamped to 11.
    fn from(quality: u8) -> Self {
        Self { value: quality.min(11) }
    }
}

impl FromStr for BrotliQuality {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s.parse::<u8>()?))
    }
}

impl From<BrotliQuality> for u8 {
    fn from(quality: BrotliQuality) -> u8 {
        quality.value
    }
}

impl From<BrotliQuality> for i32 {
    fn from(quality: BrotliQuality) -> i32 {
        quality.value as i32
    }
}
