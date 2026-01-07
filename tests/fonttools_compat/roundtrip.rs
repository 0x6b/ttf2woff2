use std::{fs::read, path::PathBuf};

use ttf2woff2::{BrotliQuality, encode};

use crate::{read_test_font, test_dir};
use std::{
    fs::{read, write},
    process::Command,
};
use ttf2woff2::{encode, encode_no_transform, BrotliQuality};

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

    let valid = validate_with_fonttools(ttf_path.to_str().unwrap(), woff2_path.to_str().unwrap());

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

    let rust_woff2 = encode(&ttf_data, BrotliQuality::default()).unwrap();

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

    assert!(
        size_diff_pct.abs() < 5.0,
        "WOFF2 size differs too much from fonttools: {:.2}%",
        size_diff_pct
    );
}
