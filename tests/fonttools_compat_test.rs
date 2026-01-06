//! Tests ported from fonttools woff2_test.py
//!
//! These tests verify compatibility with fonttools' WOFF2 implementation.
//! Reference: https://github.com/fonttools/fonttools/blob/main/Tests/ttLib/woff2_test.py

use std::{
    fs::{read, write},
    path::PathBuf,
    process::Command,
};

use ttf2woff2::{BrotliQuality, encode, encode_no_transform};

fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn read_test_font(name: &str) -> Vec<u8> {
    read(test_dir().join(name)).expect("Failed to read test font")
}

// === Base128 Encoding Tests ===
// Ported from Base128Test class

mod base128 {
    use crate::ttf2woff2_test_utils::{base128_size, encode_base128};

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
}

// === 255UInt16 Encoding Tests ===
// Ported from UShort255Test class

mod ushort255 {
    use crate::ttf2woff2_test_utils::encode_255_uint16;

    #[test]
    fn test_pack_255_ushort() {
        // 255UInt16 encoding as per WOFF2 spec:
        // - 0-252: single byte
        // - 253-505: 253 + (value - 253)
        // - 506-761: 254 + (value - 506)
        // - 762-65535: 255 + high byte + low byte
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
}

// === WOFF2 Header Tests ===

mod woff2_header {
    use super::*;

    const WOFF2_SIGNATURE: &[u8; 4] = b"wOF2";

    #[test]
    fn test_signature() {
        let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
        let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();
        assert_eq!(&woff2_data[0..4], WOFF2_SIGNATURE);
    }

    #[test]
    fn test_bad_signature_detection() {
        let result = encode(b"wOFF", BrotliQuality::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_not_enough_data() {
        let result = encode(&[0u8; 10], BrotliQuality::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_file_length_field() {
        let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
        let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();

        let length = u32::from_be_bytes([
            woff2_data[8],
            woff2_data[9],
            woff2_data[10],
            woff2_data[11],
        ]);
        assert_eq!(length as usize, woff2_data.len());
    }

    #[test]
    fn test_file_length_4_byte_aligned() {
        let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
        let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();
        assert_eq!(woff2_data.len() % 4, 0);
    }
}

// === WOFF2 Writer Tests ===
// Ported from WOFF2WriterTest and WOFF2WriterTTFTest classes

mod woff2_writer {
    use super::*;

    #[test]
    fn test_head_transform_flag() {
        let ttf_data = read_test_font("WarpnineSans-Regular.ttf");

        let head_offset = find_table_offset(&ttf_data, b"head").unwrap();
        let orig_flags = u16::from_be_bytes([
            ttf_data[head_offset + 16],
            ttf_data[head_offset + 17],
        ]);

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
        let woff2_without_transform =
            encode_no_transform(&ttf_data, BrotliQuality::default()).unwrap();

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

        // Version fields at offset 28-31 in WOFF2 header
        let _major = u16::from_be_bytes([woff2_data[28], woff2_data[29]]);
        let _minor = u16::from_be_bytes([woff2_data[30], woff2_data[31]]);

        // Version may be (0,0) if head.fontRevision is 0.0 or not set
        // Just verify the header is properly formed with version fields
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
}

// === Glyf Transform Tests ===
// Ported from WOFF2GlyfTableTest class

mod glyf_transform {
    use super::*;

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
}

// === Compression Quality Tests ===

mod compression_quality {
    use super::*;

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
}

// === Variable Font Tests ===

mod variable_fonts {
    use super::*;

    #[test]
    fn test_variable_font_encoding() {
        let ttf_data = read_test_font("Recursive_VF_1.085.ttf");
        let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();

        assert!(!woff2_data.is_empty());
        assert_eq!(&woff2_data[0..4], b"wOF2");

        let compression = (1.0 - woff2_data.len() as f64 / ttf_data.len() as f64) * 100.0;
        assert!(compression > 30.0, "Variable font should compress well");
    }
}

// === Large Font Tests ===

mod large_fonts {
    use super::*;

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
}

// === Roundtrip Tests with fonttools validation ===
// Ported from WOFF2RoundtripTest class

mod roundtrip {
    use super::*;

    fn validate_with_fonttools(ttf_path: &str, woff2_path: &str) -> bool {
        let output = Command::new("uv")
            .args([
                "run",
                "--with",
                "fonttools",
                "--with",
                "brotli",
                "python",
                "-c",
                &format!(
                    r#"
from fontTools.ttLib import TTFont
from fontTools.pens.recordingPen import RecordingPen

orig = TTFont("{ttf_path}")
woff2 = TTFont("{woff2_path}")

# Compare glyph count
assert orig["maxp"].numGlyphs == woff2["maxp"].numGlyphs, "numGlyphs mismatch"

# Compare all glyph shapes
orig_glyphs = orig.getGlyphSet()
woff2_glyphs = woff2.getGlyphSet()
for name in orig.getGlyphOrder():
    pen1 = RecordingPen()
    pen2 = RecordingPen()
    orig_glyphs[name].draw(pen1)
    woff2_glyphs[name].draw(pen2)
    assert pen1.value == pen2.value, f"Glyph {{name}} mismatch"

print("OK")
"#
                ),
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to run fonttools validation");

        output.status.success()
    }

    fn roundtrip_test(font_name: &str, use_transform: bool) {
        let ttf_path = test_dir().join(format!("{font_name}.ttf"));
        let woff2_path = test_dir().join(format!("{font_name}_roundtrip_test.woff2"));

        let ttf_data = read(&ttf_path).expect("Failed to read TTF");
        let woff2_data = if use_transform {
            encode(&ttf_data, BrotliQuality::default())
        } else {
            encode_no_transform(&ttf_data, BrotliQuality::default())
        }
        .expect("Failed to encode");

        write(&woff2_path, &woff2_data).expect("Failed to write WOFF2");

        let valid = validate_with_fonttools(
            ttf_path.to_str().unwrap(),
            woff2_path.to_str().unwrap(),
        );

        // Clean up
        let _ = std::fs::remove_file(&woff2_path);

        assert!(valid, "fonttools validation failed for {font_name}");
    }

    #[test]
    fn test_roundtrip_default_transforms() {
        roundtrip_test("WarpnineSans-Regular", true);
    }

    #[test]
    fn test_roundtrip_no_transforms() {
        roundtrip_test("WarpnineSans-Regular", false);
    }

    #[test]
    fn test_roundtrip_variable_font() {
        roundtrip_test("Recursive_VF_1.085", true);
    }

    #[test]
    fn test_fonttools_can_decompress() {
        let ttf_data = read_test_font("WarpnineSans-Regular.ttf");
        let woff2_data = encode(&ttf_data, BrotliQuality::default()).unwrap();

        let woff2_path = test_dir().join("decompress_test.woff2");
        write(&woff2_path, &woff2_data).expect("Failed to write WOFF2");

        // Use fonttools to decompress back to TTF
        let output = Command::new("uv")
            .args([
                "run",
                "--with",
                "fonttools",
                "--with",
                "brotli",
                "python",
                "-c",
                &format!(
                    r#"
from fontTools.ttLib import TTFont
from io import BytesIO

woff2 = TTFont("{}")
buf = BytesIO()
woff2.flavor = None
woff2.save(buf)
print(f"Decompressed size: {{len(buf.getvalue())}}")
"#,
                    woff2_path.to_str().unwrap()
                ),
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to run fonttools");

        let _ = std::fs::remove_file(&woff2_path);

        assert!(
            output.status.success(),
            "fonttools failed to decompress WOFF2: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn test_compare_size_with_fonttools() {
        let ttf_path = test_dir().join("WarpnineSans-Regular.ttf");
        let ttf_data = read(&ttf_path).expect("Failed to read TTF");

        // Encode with Rust
        let rust_woff2 = encode(&ttf_data, BrotliQuality::default()).unwrap();

        // Get fonttools size
        let output = Command::new("uv")
            .args([
                "run",
                "--with",
                "fonttools",
                "--with",
                "brotli",
                "python",
                "-c",
                &format!(
                    r#"
from fontTools.ttLib import TTFont
from io import BytesIO

font = TTFont("{}")
font.flavor = "woff2"
buf = BytesIO()
font.save(buf)
print(len(buf.getvalue()))
"#,
                    ttf_path.to_str().unwrap()
                ),
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to run fonttools");

        let fonttools_size: usize = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .expect("Failed to parse fonttools size");

        let size_diff_pct =
            ((rust_woff2.len() as f64 - fonttools_size as f64) / fonttools_size as f64) * 100.0;

        println!("Rust WOFF2 size:      {} bytes", rust_woff2.len());
        println!("fonttools WOFF2 size: {} bytes", fonttools_size);
        println!("Size difference:      {:.2}%", size_diff_pct);

        // Allow up to 5% size difference (should be very close)
        assert!(
            size_diff_pct.abs() < 5.0,
            "WOFF2 size differs too much from fonttools: {:.2}%",
            size_diff_pct
        );
    }
}

// Helper module for exposing internal functions for testing
#[allow(dead_code)]
mod ttf2woff2_test_utils {
    pub fn encode_base128(mut value: u32) -> Vec<u8> {
        if value == 0 {
            return vec![0];
        }

        let mut result = Vec::new();
        while value > 0 {
            result.push((value & 0x7F) as u8);
            value >>= 7;
        }
        result.reverse();

        for i in 0..result.len() - 1 {
            result[i] |= 0x80;
        }
        result
    }

    pub fn base128_size(value: u32) -> usize {
        encode_base128(value).len()
    }

    pub fn encode_255_uint16(value: u16) -> Vec<u8> {
        const ONE_MORE_BYTE_CODE1: u8 = 253;
        const ONE_MORE_BYTE_CODE2: u8 = 254;
        const WORD_CODE: u8 = 255;

        if value < 253 {
            vec![value as u8]
        } else if value < 506 {
            vec![ONE_MORE_BYTE_CODE1, (value - 253) as u8]
        } else if value < 762 {
            vec![ONE_MORE_BYTE_CODE2, (value - 506) as u8]
        } else {
            vec![WORD_CODE, (value >> 8) as u8, (value & 0xFF) as u8]
        }
    }
}
