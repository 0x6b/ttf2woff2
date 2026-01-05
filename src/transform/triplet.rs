/// Triplet encoding result - up to 4 bytes, stored inline to avoid heap allocation
#[derive(Clone, Copy)]
pub struct TripletData {
    data: [u8; 4],
    len: u8,
}

impl TripletData {
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
}

/// Encode a coordinate delta pair using WOFF2 triplet encoding.
/// Returns (flag_byte, triplet_bytes).
#[inline]
pub fn encode_triplet(x: i16, y: i16, on_curve: bool) -> (u8, TripletData) {
    let abs_x = x.unsigned_abs();
    let abs_y = y.unsigned_abs();
    let on_curve_bit: u8 = if on_curve { 0 } else { 128 };
    let x_sign: u8 = if x < 0 { 0 } else { 1 };
    let y_sign: u8 = if y < 0 { 0 } else { 1 };
    let xy_signs = x_sign + 2 * y_sign;

    // Case 1: x == 0 && abs_y < 1280
    if x == 0 && abs_y < 1280 {
        let flag = on_curve_bit + ((abs_y & 0xF00) >> 7) as u8 + y_sign;
        return (
            flag,
            TripletData { data: [(abs_y & 0xFF) as u8, 0, 0, 0], len: 1 },
        );
    }

    // Case 2: y == 0 && abs_x < 1280
    if y == 0 && abs_x < 1280 {
        let flag = on_curve_bit + 10 + ((abs_x & 0xF00) >> 7) as u8 + x_sign;
        return (
            flag,
            TripletData { data: [(abs_x & 0xFF) as u8, 0, 0, 0], len: 1 },
        );
    }

    // Case 3: abs_x in 1..65 && abs_y in 1..65
    if (1..65).contains(&abs_x) && (1..65).contains(&abs_y) {
        let flag = on_curve_bit
            + 20
            + (((abs_x - 1) & 0x30) as u8)
            + ((((abs_y - 1) & 0x30) >> 2) as u8)
            + xy_signs;
        let triplet = ((((abs_x - 1) & 0xF) << 4) | ((abs_y - 1) & 0xF)) as u8;
        return (flag, TripletData { data: [triplet, 0, 0, 0], len: 1 });
    }

    // Case 4: abs_x in 1..769 && abs_y in 1..769
    if (1..769).contains(&abs_x) && (1..769).contains(&abs_y) {
        let flag = on_curve_bit
            + 84
            + (12 * (((abs_x - 1) & 0x300) >> 8)) as u8
            + ((((abs_y - 1) & 0x300) >> 6) as u8)
            + xy_signs;
        return (
            flag,
            TripletData {
                data: [((abs_x - 1) & 0xFF) as u8, ((abs_y - 1) & 0xFF) as u8, 0, 0],
                len: 2,
            },
        );
    }

    // Case 5: abs_x < 4096 && abs_y < 4096
    if abs_x < 4096 && abs_y < 4096 {
        let flag = on_curve_bit + 120 + xy_signs;
        return (
            flag,
            TripletData {
                data: [
                    (abs_x >> 4) as u8,
                    (((abs_x & 0xF) << 4) | (abs_y >> 8)) as u8,
                    (abs_y & 0xFF) as u8,
                    0,
                ],
                len: 3,
            },
        );
    }

    // Case 6: Full 16-bit range
    let flag = on_curve_bit + 124 + xy_signs;
    (
        flag,
        TripletData {
            data: [
                (abs_x >> 8) as u8,
                (abs_x & 0xFF) as u8,
                (abs_y >> 8) as u8,
                (abs_y & 0xFF) as u8,
            ],
            len: 4,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case1_x_zero() {
        let (flag, data) = encode_triplet(0, 100, true);
        assert_eq!(flag, 0 + 0 + 1);
        assert_eq!(data.as_slice(), &[100]);
    }

    #[test]
    fn test_case2_y_zero() {
        let (flag, data) = encode_triplet(100, 0, true);
        assert_eq!(flag, 0 + 10 + 0 + 1);
        assert_eq!(data.as_slice(), &[100]);
    }

    #[test]
    fn test_case3_small_both() {
        let (flag, data) = encode_triplet(10, 20, true);
        assert!(flag >= 20 && flag < 84);
        assert_eq!(data.as_slice().len(), 1);
    }

    #[test]
    fn test_case6_large() {
        let (flag, data) = encode_triplet(5000, 6000, true);
        assert_eq!(flag, 124 + 3);
        assert_eq!(data.as_slice().len(), 4);
    }

    #[test]
    fn test_off_curve() {
        let (flag, _) = encode_triplet(0, 100, false);
        assert!(flag & 128 != 0);
    }
}
