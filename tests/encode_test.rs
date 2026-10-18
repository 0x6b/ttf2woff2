use std::{fs::read, path::PathBuf};

use ttf2woff2::{BrotliQuality, EncodeOptions, encode_with_options};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn golden_dir() -> PathBuf {
    fixtures_dir().join("golden")
}

fn test_font(name: &str) {
    let ttf_path = fixtures_dir().join(format!("{name}.ttf"));
    let golden_path = golden_dir().join(format!("{name}.woff2"));

    let ttf_data = read(&ttf_path).expect("Failed to read TTF file");
    let golden_woff2 = read(&golden_path).expect("Failed to read golden WOFF2 file");

    // Compare like-for-like with fontTools' transformed golden output.
    let woff2_data = encode_with_options(
        &ttf_data,
        EncodeOptions {
            quality: BrotliQuality::from(9),
            transform_glyf_loca: true,
            ..Default::default()
        },
    )
    .expect("Failed to encode");

    assert!(!woff2_data.is_empty());
    assert_eq!(&woff2_data[0..4], b"wOF2", "Invalid WOFF2 signature");

    let compression = (1.0 - woff2_data.len() as f64 / ttf_data.len() as f64) * 100.0;
    println!(
        "{}: {} -> {} bytes ({:.1}% compression)",
        name,
        ttf_data.len(),
        woff2_data.len(),
        compression
    );

    // Compare against golden fixture (generated with quality 11)
    // Allow larger tolerance since we use faster quality for tests
    let size_diff_pct =
        ((woff2_data.len() as f64 - golden_woff2.len() as f64) / golden_woff2.len() as f64) * 100.0;

    println!("  vs fonttools: {} bytes ({:+.2}%)", golden_woff2.len(), size_diff_pct);

    assert!(
        size_diff_pct.abs() < 12.0,
        "WOFF2 size differs too much from fonttools golden: {:.2}%",
        size_diff_pct
    );
}

#[test]
fn test_warpnine_sans() {
    test_font("WarpnineSans-Regular");
}

#[test]
fn test_noto_sans_jp() {
    test_font("NotoSansJP-Medium");
}

#[test]
fn test_recursive_vf() {
    test_font("Recursive_VF_1.085");
}

#[test]
fn test_minimal_cff() {
    let otf_path = fixtures_dir().join("MinimalCFF-Regular.otf");
    let golden_path = golden_dir().join("MinimalCFF-Regular.woff2");

    let otf_data = read(&otf_path).expect("Failed to read OTF file");
    let golden_woff2 = read(&golden_path).expect("Failed to read golden WOFF2 file");

    // The WOFF2 spec defines no CFF transformation, so all tables are stored
    // as-is — like-for-like with fonttools' golden, which leaves CFF
    // untransformed by default.
    let woff2_data = encode_with_options(
        &otf_data,
        EncodeOptions {
            quality: BrotliQuality::from(9),
            ..Default::default()
        },
    )
    .expect("Failed to encode");

    assert!(!woff2_data.is_empty());
    assert_eq!(&woff2_data[0..4], b"wOF2", "Invalid WOFF2 signature");

    let compression = (1.0 - woff2_data.len() as f64 / otf_data.len() as f64) * 100.0;
    println!(
        "MinimalCFF-Regular: {} -> {} bytes ({:.1}% compression)",
        otf_data.len(),
        woff2_data.len(),
        compression
    );

    let size_diff_pct =
        ((woff2_data.len() as f64 - golden_woff2.len() as f64) / golden_woff2.len() as f64) * 100.0;

    println!("  vs fonttools: {} bytes ({:+.2}%)", golden_woff2.len(), size_diff_pct);

    assert!(
        size_diff_pct.abs() < 12.0,
        "WOFF2 size differs too much from fonttools golden: {:.2}%",
        size_diff_pct
    );
}
