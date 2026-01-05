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
}
