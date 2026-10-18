use ttf2woff2::{BrotliQuality, EncodeOptions, encode, encode_with_options};

use crate::read_test_font;

const OTF_FIXTURE: &str = "MinimalCFF-Regular.otf";
const WOFF2_SIGNATURE: &[u8; 4] = b"wOF2";
// Index of "CFF " in the WOFF2 spec's known-table-tags list.
const CFF_KNOWN_TAG_INDEX: u8 = 13;

struct DirectoryEntry {
    tag: Option<[u8; 4]>,
    known_tag_index: Option<u8>,
    transform_version: u8,
}

/// Decode a UIntBase128 value, returning `(value, bytes consumed)`.
fn decode_base128(data: &[u8]) -> Option<(u32, usize)> {
    let mut value: u32 = 0;
    for (i, &b) in data.iter().enumerate() {
        if value > (u32::MAX >> 7) {
            return None;
        }
        value = (value << 7) | u32::from(b & 0x7F);
        if b & 0x80 == 0 {
            return Some((value, i + 1));
        }
    }
    None
}

/// Parse the WOFF2 table directory into per-entry flag information.
fn parse_table_directory(woff2: &[u8], num_tables: usize) -> Vec<DirectoryEntry> {
    let mut pos = 48usize; // WOFF2 header size
    let mut entries = Vec::with_capacity(num_tables);
    for _ in 0..num_tables {
        let flags = woff2[pos];
        pos += 1;

        let known_tag_index = flags & 0x3F;
        let tag = if known_tag_index == 63 {
            let mut tag = [0u8; 4];
            tag.copy_from_slice(&woff2[pos..pos + 4]);
            pos += 4;
            Some(tag)
        } else {
            None
        };

        let (_, consumed) = decode_base128(&woff2[pos..]).expect("invalid origLength");
        pos += consumed;

        // Only glyf/loca with transformVersion 0 carry a transformLength field.
        let is_glyf_or_loca = tag.is_none() && matches!(known_tag_index, 10 | 11);
        if is_glyf_or_loca && flags >> 6 == 0 {
            let (_, consumed) = decode_base128(&woff2[pos..]).expect("invalid transformLength");
            pos += consumed;
        }

        entries.push(DirectoryEntry {
            tag,
            known_tag_index: (known_tag_index != 63).then_some(known_tag_index),
            transform_version: flags >> 6,
        });
    }
    entries
}

#[test]
fn test_otf_encode() {
    let otf_data = read_test_font(OTF_FIXTURE);
    let woff2_data = encode(&otf_data, BrotliQuality::from(9)).unwrap();
    assert!(!woff2_data.is_empty());
    assert_eq!(&woff2_data[0..4], WOFF2_SIGNATURE);
}

#[test]
fn test_otf_flavor_preserved() {
    let otf_data = read_test_font(OTF_FIXTURE);
    let woff2_data = encode(&otf_data, BrotliQuality::from(9)).unwrap();

    let flavor = u32::from_be_bytes([woff2_data[4], woff2_data[5], woff2_data[6], woff2_data[7]]);
    assert_eq!(flavor, 0x4F54544F, "WOFF2 flavor field must be OTTO");

    let woff2_num_tables = u16::from_be_bytes([woff2_data[12], woff2_data[13]]);
    let otf_num_tables = u16::from_be_bytes([otf_data[4], otf_data[5]]);
    assert_eq!(woff2_num_tables, otf_num_tables);
}

#[test]
fn test_cff_table_not_transformed() {
    let otf_data = read_test_font(OTF_FIXTURE);
    let woff2_data = encode(&otf_data, BrotliQuality::from(9)).unwrap();

    let num_tables = u16::from_be_bytes([woff2_data[12], woff2_data[13]]) as usize;
    let entries = parse_table_directory(&woff2_data, num_tables);

    assert_eq!(entries.len(), num_tables);

    // The WOFF2 spec defines no transformation for CFF: every table of a
    // CFF-flavored font must use transformVersion 0 (null transform).
    for entry in &entries {
        assert_eq!(entry.transform_version, 0, "table {:?} must not be transformed", entry.tag);
    }

    let cff_entries = entries
        .iter()
        .filter(|e| e.known_tag_index == Some(CFF_KNOWN_TAG_INDEX))
        .count();
    assert_eq!(cff_entries, 1, "expected exactly one CFF table entry");
}

#[test]
fn test_otf_transform_option_is_noop() {
    let otf_data = read_test_font(OTF_FIXTURE);

    let with_transform = encode_with_options(
        &otf_data,
        EncodeOptions {
            quality: BrotliQuality::from(9),
            transform_glyf_loca: true,
            ..Default::default()
        },
    )
    .unwrap();
    let without_transform = encode(&otf_data, BrotliQuality::from(9)).unwrap();

    // An OTF font has no glyf/loca, so the transform option has no effect.
    assert_eq!(with_transform, without_transform);
}
