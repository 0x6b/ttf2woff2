use std::{
    fs::{read, write},
    path::PathBuf,
    process::Command,
};

use ttf2woff2::{BrotliQuality, encode};

fn test_font(name: &str) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let ttf_path = root.join(format!("{name}.ttf"));
    let rust_woff2_path = root.join(format!("{name}.woff2"));

    // 1. Encode with Rust
    let ttf_data = read(&ttf_path).expect("Failed to read TTF file");
    let woff2_data = encode(&ttf_data, BrotliQuality::default()).expect("Failed to encode");

    assert!(!woff2_data.is_empty());
    assert_eq!(&woff2_data[0..4], b"wOF2", "Invalid WOFF2 signature");

    write(&rust_woff2_path, &woff2_data).expect("Failed to write WOFF2 file");

    let compression = (1.0 - woff2_data.len() as f64 / ttf_data.len() as f64) * 100.0;
    println!(
        "{}: {} -> {} bytes ({:.1}% compression)",
        name,
        ttf_data.len(),
        woff2_data.len(),
        compression
    );

    // 2. Validate against fonttools
    let output = Command::new("uv")
        .args([
            "run",
            "--with",
            "fonttools",
            "--with",
            "brotli",
            "scripts/validate.py",
            ttf_path.to_str().unwrap(),
            rust_woff2_path.to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run validation script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("{stdout}");
    if !stderr.is_empty() {
        eprintln!("{stderr}");
    }

    assert!(output.status.success(), "Validation against fonttools failed");
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
