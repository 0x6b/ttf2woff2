use crate::Error;

#[derive(Debug, Clone, Copy)]
pub struct BrotliQuality {
    pub value: u8,
}

impl BrotliQuality {
    pub fn try_new(quality: u8) -> Result<Self, Error> {
        if quality > 11 {
            return Err(Error::InvalidBrotliQuality(quality));
        }
        Ok(Self { value: quality })
    }

    pub fn as_i32(&self) -> i32 {
        self.value as i32
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
        Self::try_new(s.parse()?)
    }
}
