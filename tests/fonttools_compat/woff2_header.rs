use ttf2woff2::{BrotliQuality, encode};

use crate::read_test_font;

const WOFF2_SIGNATURE: &[u8; 4] = b"wOF2";

#[test]
fn test_signature() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
    let woff2_data = encode(&ttf_data, BrotliQuality::from(9)).unwrap();
    assert_eq!(&woff2_data[0..4], WOFF2_SIGNATURE);
}

#[test]
fn test_bad_signature_detection() {
    let result = encode(b"wOFF", BrotliQuality::from(9));
    assert!(result.is_err());
}

#[test]
fn test_not_enough_data() {
    let result = encode(&[0u8; 10], BrotliQuality::from(9));
    assert!(result.is_err());
}

#[test]
fn test_file_length_field() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
    let woff2_data = encode(&ttf_data, BrotliQuality::from(9)).unwrap();

    let length = u32::from_be_bytes([woff2_data[8], woff2_data[9], woff2_data[10], woff2_data[11]]);
    assert_eq!(length as usize, woff2_data.len());
}

#[test]
fn test_file_length_4_byte_aligned() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
    let woff2_data = encode(&ttf_data, BrotliQuality::from(9)).unwrap();
    assert_eq!(woff2_data.len() % 4, 0);
}
