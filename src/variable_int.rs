/// UIntBase128 encoding per WOFF2 spec.
/// Each byte uses 7 bits for data, MSB is continuation flag.
pub fn encode_base128(mut value: u32) -> Vec<u8> {
    if value == 0 {
        return vec![0x00];
    }

    let mut result = Vec::with_capacity(5);
    while value > 0 {
        result.push((value & 0x7F) as u8);
        value >>= 7;
    }
    result.reverse();

    for i in 0..result.len() - 1 {
        result[i] |= 0x80;
    }

    result
}

/// Decode UIntBase128, returns (value, bytes_consumed) or None on error.
pub fn decode_base128(data: &[u8]) -> Option<(u32, usize)> {
    let mut result: u32 = 0;
    let mut bytes_read = 0;

    for &byte in data.iter().take(5) {
        bytes_read += 1;

        if bytes_read > 1 && result == 0 && (byte & 0x7F) == 0 {
            return None;
        }

        result = result.checked_mul(128)?;
        result = result.checked_add((byte & 0x7F) as u32)?;

        if byte & 0x80 == 0 {
            return Some((result, bytes_read));
        }
    }

    None
}

/// 255UInt16 encoding per WOFF2 spec.
pub fn encode_255_u_int16(value: u16) -> Vec<u8> {
    const ONE_MORE_BYTE_CODE1: u8 = 253;
    const ONE_MORE_BYTE_CODE2: u8 = 254;
    const WORD_CODE: u8 = 255;

    if value < 253 {
        vec![value as u8]
    } else if value < 506 {
        vec![ONE_MORE_BYTE_CODE1, (value - 253) as u8]
    } else if value < 762 {
        vec![ONE_MORE_BYTE_CODE2, (value - 506) as u8]
    } else {
        vec![WORD_CODE, (value >> 8) as u8, (value & 0xFF) as u8]
    }
}

/// Decode 255UInt16, returns (value, bytes_consumed) or None on error.
pub fn decode_255_u_int16(data: &[u8]) -> Option<(u16, usize)> {
    let first = *data.first()?;

    match first {
        253 => {
            let second = *data.get(1)?;
            Some((253 + second as u16, 2))
        }
        254 => {
            let second = *data.get(1)?;
            Some((506 + second as u16, 2))
        }
        255 => {
            let high = *data.get(1)? as u16;
            let low = *data.get(2)? as u16;
            Some(((high << 8) | low, 3))
        }
        _ => Some((first as u16, 1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base128() {
        assert_eq!(encode_base128(0), vec![0x00]);
        assert_eq!(encode_base128(63), vec![0x3F]);
        assert_eq!(encode_base128(16383), vec![0xFF, 0x7F]);
        assert_eq!(encode_base128(127), vec![0x7F]);
        assert_eq!(encode_base128(128), vec![0x81, 0x00]);
    }

    #[test]
    fn test_decode_base128() {
        assert_eq!(decode_base128(&[0x00]), Some((0, 1)));
        assert_eq!(decode_base128(&[0x3F]), Some((63, 1)));
        assert_eq!(decode_base128(&[0xFF, 0x7F]), Some((16383, 2)));
        assert_eq!(decode_base128(&[0x81, 0x00]), Some((128, 2)));
    }

    #[test]
    fn test_base128_roundtrip() {
        for value in [0, 1, 127, 128, 255, 16383, 16384, 65535, 0x0FFF_FFFF] {
            let encoded = encode_base128(value);
            let (decoded, _) = decode_base128(&encoded).unwrap();
            assert_eq!(decoded, value, "roundtrip failed for {value}");
        }
    }

    #[test]
    fn test_encode_255_u_int16() {
        assert_eq!(encode_255_u_int16(0), vec![0]);
        assert_eq!(encode_255_u_int16(252), vec![252]);
        assert_eq!(encode_255_u_int16(253), vec![253, 0]);
        assert_eq!(encode_255_u_int16(505), vec![253, 252]);
        assert_eq!(encode_255_u_int16(506), vec![254, 0]);
        assert_eq!(encode_255_u_int16(761), vec![254, 255]);
        assert_eq!(encode_255_u_int16(762), vec![255, 0x02, 0xFA]);
        assert_eq!(encode_255_u_int16(0xFFFF), vec![255, 0xFF, 0xFF]);
    }

    #[test]
    fn test_decode_255_u_int16() {
        assert_eq!(decode_255_u_int16(&[0]), Some((0, 1)));
        assert_eq!(decode_255_u_int16(&[253, 0]), Some((253, 2)));
        assert_eq!(decode_255_u_int16(&[254, 0]), Some((506, 2)));
        assert_eq!(decode_255_u_int16(&[255, 0x02, 0xFA]), Some((762, 3)));
    }

    #[test]
    fn test_255_u_int16_roundtrip() {
        for value in [0, 1, 252, 253, 505, 506, 761, 762, 1000, 0xFFFF] {
            let encoded = encode_255_u_int16(value);
            let (decoded, _) = decode_255_u_int16(&encoded).unwrap();
            assert_eq!(decoded, value, "roundtrip failed for {value}");
        }
    }
}
