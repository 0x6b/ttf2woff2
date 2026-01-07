//! Tests ported from fonttools woff2_test.py
//!
//! These tests verify compatibility with fonttools' WOFF2 implementation.
//! Reference: https://github.com/fonttools/fonttools/blob/main/Tests/ttLib/woff2_test.py

use std::{fs::read, path::PathBuf};

mod base128;
mod compression_quality;
mod glyf_transform;
mod large_fonts;
mod roundtrip;
mod ushort255;
mod variable_fonts;
mod woff2_header;
mod woff2_writer;

pub fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn read_test_font(name: &str) -> Vec<u8> {
    read(test_dir().join(name)).expect("Failed to read test font")
}

#[allow(dead_code)]
pub mod test_utils {
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
