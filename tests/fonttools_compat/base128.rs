use crate::test_utils::{base128_size, encode_base128};

#[test]
fn test_encode_base128_values() {
    assert_eq!(encode_base128(63), vec![0x3F]);
    assert_eq!(encode_base128(u32::MAX), vec![0x8F, 0xFF, 0xFF, 0xFF, 0x7F]);
    assert_eq!(encode_base128(0), vec![0x00]);
    assert_eq!(encode_base128(127), vec![0x7F]);
    assert_eq!(encode_base128(128), vec![0x81, 0x00]);
    assert_eq!(encode_base128(16383), vec![0xFF, 0x7F]);
}

#[test]
fn test_base128_size() {
    assert_eq!(base128_size(0), 1);
    assert_eq!(base128_size(24567), 3);
    assert_eq!(base128_size(u32::MAX), 5);
}
