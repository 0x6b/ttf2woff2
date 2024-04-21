use crate::Error;

/// Brotli compression quality
///
/// The quality parameter is an integer from 0 to 11.
#[derive(Debug, Clone, Copy)]
pub struct BrotliQuality {
    pub value: u8,
}

impl BrotliQuality {
    /// Create a new BrotliQuality
    ///
    /// If the quality is greater than 11, it will be clamped to 11.
    pub fn new(quality: u8) -> Self {
        Self { value: if quality > 11 { 11 } else { quality } }
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
        Ok(Self::new(s.parse()?))
    }
}

impl From<BrotliQuality> for i32 {
    fn from(quality: BrotliQuality) -> i32 {
        quality.value as i32
    }
}
