use ttf2woff2::{BrotliQuality, encode};

use crate::read_test_font;

#[test]
fn test_different_quality_levels() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");

    let q5 = encode(&ttf_data, BrotliQuality::from(5)).unwrap();
    let q9 = encode(&ttf_data, BrotliQuality::from(9)).unwrap();
    let q11 = encode(&ttf_data, BrotliQuality::default()).unwrap();

    assert!(q5.len() >= q9.len());
    assert!(q9.len() >= q11.len());
}

#[test]
fn test_quality_clamps_to_max() {
    let q11 = BrotliQuality::from(11);
    let q12 = BrotliQuality::from(12);
    let q255 = BrotliQuality::from(255);

    assert_eq!(q11.value, 11);
    assert_eq!(q12.value, 11); // clamped
    assert_eq!(q255.value, 11); // clamped
}
