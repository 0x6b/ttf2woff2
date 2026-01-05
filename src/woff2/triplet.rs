use super::inline_bytes::InlineBytes;

/// Input for triplet encoding: (dx, dy, on_curve)
pub(super) struct TripletInput {
    pub dx: i16,
    pub dy: i16,
    pub on_curve: bool,
}

/// Encoded triplet result
pub(super) struct EncodedTriplet {
    pub flag: u8,
    pub data: InlineBytes<4>,
}

impl From<TripletInput> for EncodedTriplet {
    #[inline]
    fn from(input: TripletInput) -> Self {
        let TripletInput { dx: x, dy: y, on_curve } = input;
        let abs_x = x.unsigned_abs();
        let abs_y = y.unsigned_abs();
        let on_curve_bit: u8 = if on_curve { 0 } else { 128 };
        let x_sign: u8 = if x < 0 { 0 } else { 1 };
        let y_sign: u8 = if y < 0 { 0 } else { 1 };
        let xy_signs = x_sign + 2 * y_sign;

        // Case 1: x == 0 && abs_y < 1280
        if x == 0 && abs_y < 1280 {
            let flag = on_curve_bit + ((abs_y & 0xF00) >> 7) as u8 + y_sign;
            return EncodedTriplet {
                flag,
                data: InlineBytes::new([(abs_y & 0xFF) as u8, 0, 0, 0], 1),
            };
        }

        // Case 2: y == 0 && abs_x < 1280
        if y == 0 && abs_x < 1280 {
            let flag = on_curve_bit + 10 + ((abs_x & 0xF00) >> 7) as u8 + x_sign;
            return EncodedTriplet {
                flag,
                data: InlineBytes::new([(abs_x & 0xFF) as u8, 0, 0, 0], 1),
            };
        }

        // Case 3: abs_x in 1..65 && abs_y in 1..65
        if (1..65).contains(&abs_x) && (1..65).contains(&abs_y) {
            let flag = on_curve_bit
                + 20
                + (((abs_x - 1) & 0x30) as u8)
                + ((((abs_y - 1) & 0x30) >> 2) as u8)
                + xy_signs;
            let triplet = ((((abs_x - 1) & 0xF) << 4) | ((abs_y - 1) & 0xF)) as u8;
            return EncodedTriplet {
                flag,
                data: InlineBytes::new([triplet, 0, 0, 0], 1),
            };
        }

        // Case 4: abs_x in 1..769 && abs_y in 1..769
        if (1..769).contains(&abs_x) && (1..769).contains(&abs_y) {
            let flag = on_curve_bit
                + 84
                + (12 * (((abs_x - 1) & 0x300) >> 8)) as u8
                + ((((abs_y - 1) & 0x300) >> 6) as u8)
                + xy_signs;
            return EncodedTriplet {
                flag,
                data: InlineBytes::new(
                    [((abs_x - 1) & 0xFF) as u8, ((abs_y - 1) & 0xFF) as u8, 0, 0],
                    2,
                ),
            };
        }

        // Case 5: abs_x < 4096 && abs_y < 4096
        if abs_x < 4096 && abs_y < 4096 {
            let flag = on_curve_bit + 120 + xy_signs;
            return EncodedTriplet {
                flag,
                data: InlineBytes::new(
                    [
                        (abs_x >> 4) as u8,
                        (((abs_x & 0xF) << 4) | (abs_y >> 8)) as u8,
                        (abs_y & 0xFF) as u8,
                        0,
                    ],
                    3,
                ),
            };
        }

        // Case 6: Full 16-bit range
        let flag = on_curve_bit + 124 + xy_signs;
        EncodedTriplet {
            flag,
            data: InlineBytes::new(
                [
                    (abs_x >> 8) as u8,
                    (abs_x & 0xFF) as u8,
                    (abs_y >> 8) as u8,
                    (abs_y & 0xFF) as u8,
                ],
                4,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case1_x_zero() {
        let encoded = EncodedTriplet::from(TripletInput { dx: 0, dy: 100, on_curve: true });
        assert_eq!(encoded.flag, 0 + 0 + 1);
        assert_eq!(encoded.data.as_slice(), &[100]);
    }

    #[test]
    fn test_case2_y_zero() {
        let encoded = EncodedTriplet::from(TripletInput { dx: 100, dy: 0, on_curve: true });
        assert_eq!(encoded.flag, 0 + 10 + 0 + 1);
        assert_eq!(encoded.data.as_slice(), &[100]);
    }

    #[test]
    fn test_case3_small_both() {
        let encoded = EncodedTriplet::from(TripletInput { dx: 10, dy: 20, on_curve: true });
        assert!(encoded.flag >= 20 && encoded.flag < 84);
        assert_eq!(encoded.data.as_slice().len(), 1);
    }

    #[test]
    fn test_case6_large() {
        let encoded = EncodedTriplet::from(TripletInput { dx: 5000, dy: 6000, on_curve: true });
        assert_eq!(encoded.flag, 124 + 3);
        assert_eq!(encoded.data.as_slice().len(), 4);
    }

    #[test]
    fn test_off_curve() {
        let encoded = EncodedTriplet::from(TripletInput { dx: 0, dy: 100, on_curve: false });
        assert!(encoded.flag & 128 != 0);
    }
}
