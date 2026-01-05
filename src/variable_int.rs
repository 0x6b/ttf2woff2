use crate::inline_bytes::InlineBytes;

/// Encoded variable integer - up to 5 bytes for base128, 3 bytes for 255UInt16
pub type EncodedInt = InlineBytes<5>;

/// UIntBase128 encoding per WOFF2 spec.
#[inline]
pub fn encode_base128(mut value: u32) -> EncodedInt {
    if value == 0 {
        return InlineBytes::new([0, 0, 0, 0, 0], 1);
    }

    let mut result = [0u8; 5];
    let mut len = 0u8;

    while value > 0 {
        result[len as usize] = (value & 0x7F) as u8;
        value >>= 7;
        len += 1;
    }

    // Reverse in place
    let mut i = 0;
    let mut j = len as usize - 1;
    while i < j {
        result.swap(i, j);
        i += 1;
        j -= 1;
    }

    // Set continuation bits
    for item in result.iter_mut().take(len as usize - 1) {
        *item |= 0x80;
    }

    InlineBytes::new(result, len)
}

/// 255UInt16 encoding per WOFF2 spec.
#[inline]
pub fn encode_255_u_int16(value: u16) -> EncodedInt {
    const ONE_MORE_BYTE_CODE1: u8 = 253;
    const ONE_MORE_BYTE_CODE2: u8 = 254;
    const WORD_CODE: u8 = 255;

    if value < 253 {
        InlineBytes::new([value as u8, 0, 0, 0, 0], 1)
    } else if value < 506 {
        InlineBytes::new([ONE_MORE_BYTE_CODE1, (value - 253) as u8, 0, 0, 0], 2)
    } else if value < 762 {
        InlineBytes::new([ONE_MORE_BYTE_CODE2, (value - 506) as u8, 0, 0, 0], 2)
    } else {
        InlineBytes::new([WORD_CODE, (value >> 8) as u8, (value & 0xFF) as u8, 0, 0], 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base128() {
        assert_eq!(encode_base128(0).as_slice(), &[0x00]);
        assert_eq!(encode_base128(63).as_slice(), &[0x3F]);
        assert_eq!(encode_base128(16383).as_slice(), &[0xFF, 0x7F]);
        assert_eq!(encode_base128(127).as_slice(), &[0x7F]);
        assert_eq!(encode_base128(128).as_slice(), &[0x81, 0x00]);
    }

    #[test]
    fn test_encode_255_u_int16() {
        assert_eq!(encode_255_u_int16(0).as_slice(), &[0]);
        assert_eq!(encode_255_u_int16(252).as_slice(), &[252]);
        assert_eq!(encode_255_u_int16(253).as_slice(), &[253, 0]);
        assert_eq!(encode_255_u_int16(505).as_slice(), &[253, 252]);
        assert_eq!(encode_255_u_int16(506).as_slice(), &[254, 0]);
        assert_eq!(encode_255_u_int16(761).as_slice(), &[254, 255]);
        assert_eq!(encode_255_u_int16(762).as_slice(), &[255, 0x02, 0xFA]);
        assert_eq!(encode_255_u_int16(0xFFFF).as_slice(), &[255, 0xFF, 0xFF]);
    }
}
