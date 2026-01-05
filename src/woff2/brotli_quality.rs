use crate::Error;

/// [Brotli](https://github.com/google/brotli/) compression quality.
///
/// The quality parameter is an integer from 0 to 11.
#[derive(Debug, Clone, Copy)]
pub struct BrotliQuality {
    pub value: u8,
}

impl BrotliQuality {
    /// Create a new BrotliQuality with the default value of 11.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new BrotliQuality with the given quality.
    /// If the quality is greater than 11, it is clamped to 11.
    pub fn from(quality: u8) -> Self {
        Self { value: quality.min(11) }
    }
}

impl Default for BrotliQuality {
    fn default() -> Self {
        Self { value: 11 }
    }
}

impl std::str::FromStr for BrotliQuality {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s.parse()?))
    }
}

impl From<BrotliQuality> for i32 {
    fn from(quality: BrotliQuality) -> i32 {
        quality.value as i32
    }
}

impl From<u8> for BrotliQuality {
    fn from(quality: u8) -> Self {
        Self { value: quality.min(11) }
    }
}
