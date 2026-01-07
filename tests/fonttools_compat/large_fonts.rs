use ttf2woff2::{BrotliQuality, encode};

use crate::read_test_font;

#[test]
fn test_large_cjk_font() {
    let ttf_data = read_test_font("NotoSansJP-Medium.ttf");
    let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();

    assert!(!woff2_data.is_empty());
    assert_eq!(&woff2_data[0..4], b"wOF2");

    let compression = (1.0 - woff2_data.len() as f64 / ttf_data.len() as f64) * 100.0;
    assert!(
        compression > 50.0,
        "CJK font should achieve > 50% compression, got {:.1}%",
        compression
    );
}
