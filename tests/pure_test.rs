use std::fs;

use ttf2woff2::{BrotliQuality, pure::encode};

fn encode_font(name: &str, quality: u8) {
    let quality = BrotliQuality::from(quality);
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let input = root.join(format!("{name}.ttf"));
    let ttf_data = fs::read(&input).expect("Failed to read TTF file");

    let woff2_data = encode(&ttf_data, quality).expect("Failed to encode");

    assert!(!woff2_data.is_empty());
    assert_eq!(&woff2_data[0..4], b"wOF2");
    println!(
        "Encoded {} bytes TTF to {} bytes WOFF2 ({:.1}% compression)",
        ttf_data.len(),
        woff2_data.len(),
        (1.0 - woff2_data.len() as f64 / ttf_data.len() as f64) * 100.0
    );

    let output = root.join(format!("{name}-pure.woff2"));
    fs::write(&output, &woff2_data).expect("Failed to write WOFF2 file");
    println!("Wrote to {}", output.display());
}

#[test]
fn test_pure_encode() {
    encode_font("WarpnineSans-Regular", 11);
}

#[test]
fn test_noto() {
    encode_font("NotoSansJP-Medium", 11);
}
