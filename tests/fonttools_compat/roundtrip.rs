use std::{fs::read, path::PathBuf};

use ttf2woff2::{BrotliQuality, encode};

use crate::{read_test_font, test_dir};

fn golden_dir() -> PathBuf {
    test_dir().join("golden")
}

fn read_golden_woff2(name: &str) -> Vec<u8> {
    read(golden_dir().join(format!("{name}.woff2"))).expect("Failed to read golden WOFF2")
}

#[test]
fn test_signature_matches_fonttools() {
    let rust_woff2 =
        encode(&read_test_font("WarpnineSans-Regular.ttf"), BrotliQuality::default()).unwrap();
    let golden_woff2 = read_golden_woff2("WarpnineSans-Regular");

    assert_eq!(&rust_woff2[0..4], &golden_woff2[0..4], "WOFF2 signature mismatch");
}

#[test]
fn test_size_within_tolerance_warpnine() {
    let rust_woff2 =
        encode(&read_test_font("WarpnineSans-Regular.ttf"), BrotliQuality::default()).unwrap();
    let golden_woff2 = read_golden_woff2("WarpnineSans-Regular");

    let size_diff_pct =
        ((rust_woff2.len() as f64 - golden_woff2.len() as f64) / golden_woff2.len() as f64) * 100.0;

    println!("Rust WOFF2 size:      {} bytes", rust_woff2.len());
    println!("fonttools WOFF2 size: {} bytes", golden_woff2.len());
    println!("Size difference:      {:.2}%", size_diff_pct);

    assert!(
        size_diff_pct.abs() < 5.0,
        "WOFF2 size differs too much from fonttools: {:.2}%",
        size_diff_pct
    );
}

#[test]
fn test_size_within_tolerance_noto_cjk() {
    let rust_woff2 =
        encode(&read_test_font("NotoSansJP-Medium.ttf"), BrotliQuality::default()).unwrap();
    let golden_woff2 = read_golden_woff2("NotoSansJP-Medium");

    let size_diff_pct =
        ((rust_woff2.len() as f64 - golden_woff2.len() as f64) / golden_woff2.len() as f64) * 100.0;

    println!("Rust WOFF2 size:      {} bytes", rust_woff2.len());
    println!("fonttools WOFF2 size: {} bytes", golden_woff2.len());
    println!("Size difference:      {:.2}%", size_diff_pct);

    assert!(
        size_diff_pct.abs() < 5.0,
        "CJK font WOFF2 size differs too much from fonttools: {:.2}%",
        size_diff_pct
    );
}

#[test]
fn test_size_within_tolerance_variable_font() {
    let rust_woff2 =
        encode(&read_test_font("Recursive_VF_1.085.ttf"), BrotliQuality::default()).unwrap();
    let golden_woff2 = read_golden_woff2("Recursive_VF_1.085");

    let size_diff_pct =
        ((rust_woff2.len() as f64 - golden_woff2.len() as f64) / golden_woff2.len() as f64) * 100.0;

    println!("Rust WOFF2 size:      {} bytes", rust_woff2.len());
    println!("fonttools WOFF2 size: {} bytes", golden_woff2.len());
    println!("Size difference:      {:.2}%", size_diff_pct);

    assert!(
        size_diff_pct.abs() < 5.0,
        "Variable font WOFF2 size differs too much from fonttools: {:.2}%",
        size_diff_pct
    );
}

#[test]
fn test_num_tables_matches() {
    let rust_woff2 =
        encode(&read_test_font("WarpnineSans-Regular.ttf"), BrotliQuality::default()).unwrap();
    let golden_woff2 = read_golden_woff2("WarpnineSans-Regular");

    let rust_num_tables = u16::from_be_bytes([rust_woff2[12], rust_woff2[13]]);
    let golden_num_tables = u16::from_be_bytes([golden_woff2[12], golden_woff2[13]]);

    assert_eq!(rust_num_tables, golden_num_tables, "Number of tables mismatch");
}
