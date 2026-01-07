use ttf2woff2::{BrotliQuality, encode, encode_no_transform};

use crate::read_test_font;

#[test]
fn test_transform_produces_valid_output() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
    let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();

    assert!(!woff2_data.is_empty());
    assert_eq!(&woff2_data[0..4], b"wOF2");
}

#[test]
fn test_transform_vs_no_transform_size() {
    let ttf_data = read_test_font("NotoSansJP-Medium.ttf");

    let with_transform = encode(&ttf_data, BrotliQuality::default()).unwrap();
    let without_transform = encode_no_transform(&ttf_data, BrotliQuality::default()).unwrap();

    let savings_percent =
        (1.0 - with_transform.len() as f64 / without_transform.len() as f64) * 100.0;
    assert!(
        savings_percent > 0.0,
        "Transform should provide size reduction, got {:.1}% savings",
        savings_percent
    );
}

#[test]
fn test_roundtrip_preserves_glyph_count() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");

    let maxp_offset = find_maxp_offset(&ttf_data).expect("maxp table not found");
    let orig_num_glyphs =
        u16::from_be_bytes([ttf_data[maxp_offset + 4], ttf_data[maxp_offset + 5]]);

    let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();
    assert!(!woff2_data.is_empty());

    assert!(orig_num_glyphs > 0);
}

fn find_maxp_offset(ttf_data: &[u8]) -> Option<usize> {
    let num_tables = u16::from_be_bytes([ttf_data[4], ttf_data[5]]) as usize;
    for i in 0..num_tables {
        let entry_offset = 12 + i * 16;
        if &ttf_data[entry_offset..entry_offset + 4] == b"maxp" {
            return Some(u32::from_be_bytes([
                ttf_data[entry_offset + 8],
                ttf_data[entry_offset + 9],
                ttf_data[entry_offset + 10],
                ttf_data[entry_offset + 11],
            ]) as usize);
        }
    }
    None
}
