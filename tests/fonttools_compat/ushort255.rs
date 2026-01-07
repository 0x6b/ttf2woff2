use crate::test_utils::encode_255_uint16;

#[test]
fn test_pack_255_ushort() {
    assert_eq!(encode_255_uint16(252), vec![0xFC]);
    assert_eq!(encode_255_uint16(505), vec![0xFD, 0xFC]); // 253 + 252
    assert_eq!(encode_255_uint16(506), vec![0xFE, 0x00]); // 254 + 0
    assert_eq!(encode_255_uint16(762), vec![0xFF, 0x02, 0xFA]); // 255 + word
}

#[test]
fn test_boundary_values() {
    assert_eq!(encode_255_uint16(0), vec![0x00]);
    assert_eq!(encode_255_uint16(252), vec![0xFC]);
    assert_eq!(encode_255_uint16(253), vec![0xFD, 0x00]); // 253 + 0
    assert_eq!(encode_255_uint16(505), vec![0xFD, 0xFC]); // 253 + 252
    assert_eq!(encode_255_uint16(761), vec![0xFE, 0xFF]); // 254 + 255
    assert_eq!(encode_255_uint16(0xFFFF), vec![0xFF, 0xFF, 0xFF]);
}
