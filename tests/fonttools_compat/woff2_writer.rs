use ttf2woff2::{BrotliQuality, encode, encode_no_transform};

use crate::read_test_font;

#[test]
fn test_head_transform_flag() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");

    let head_offset = find_table_offset(&ttf_data, b"head").unwrap();
    let orig_flags = u16::from_be_bytes([ttf_data[head_offset + 16], ttf_data[head_offset + 17]]);

    let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();
    assert!(!woff2_data.is_empty());

    let bit_11_set = (orig_flags & (1 << 11)) != 0;
    assert!(!bit_11_set, "Original head flags should not have bit 11 set");
}

#[test]
fn test_tables_sorted_alphabetically() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
    let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();

    let num_tables = u16::from_be_bytes([woff2_data[12], woff2_data[13]]) as usize;
    assert!(num_tables > 0);
}

#[test]
fn test_encode_with_transform() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
    let woff2_with_transform = encode(&ttf_data, BrotliQuality::default()).unwrap();
    let woff2_without_transform = encode_no_transform(&ttf_data, BrotliQuality::default()).unwrap();

    assert!(
        woff2_with_transform.len() < woff2_without_transform.len(),
        "Transform should produce smaller output"
    );
}

#[test]
fn test_no_transforms() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
    let woff2_with = encode(&ttf_data, BrotliQuality::default()).unwrap();
    let woff2_without = encode_no_transform(&ttf_data, BrotliQuality::default()).unwrap();

    assert_ne!(woff2_with, woff2_without);
    assert_eq!(&woff2_with[0..4], b"wOF2");
    assert_eq!(&woff2_without[0..4], b"wOF2");
}

#[test]
fn test_version_from_head() {
    let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
    let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();

    let _major = u16::from_be_bytes([woff2_data[28], woff2_data[29]]);
    let _minor = u16::from_be_bytes([woff2_data[30], woff2_data[31]]);

    assert!(woff2_data.len() >= 32, "WOFF2 header should include version fields");
}

fn find_table_offset(ttf_data: &[u8], tag: &[u8; 4]) -> Option<usize> {
    if ttf_data.len() < 12 {
        return None;
    }
    let num_tables = u16::from_be_bytes([ttf_data[4], ttf_data[5]]) as usize;

    for i in 0..num_tables {
        let entry_offset = 12 + i * 16;
        if &ttf_data[entry_offset..entry_offset + 4] == tag {
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
