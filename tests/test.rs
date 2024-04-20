use std::path::Path;

use anyhow::Result;
use camino::Utf8PathBuf;
use sha2::{Digest, Sha256};
use tokio::fs::read;
use ttf2woff2::{BrotliQuality, Converter};

#[tokio::test]
async fn test() -> Result<()> {
    let root = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let input = root.join("NotoSansJP-Medium.ttf");
    let output = root.join("NotoSansJP-Medium.woff2");

    let converter = Converter::from(input, Some(output.clone()), BrotliQuality::default()).await?;
    let (input_size, output_size) = converter.write_to_woff2().await?;

    // pre-calculated SHA-256 hash and output file size using `woff2_compress` command from
    // https://github.com/google/woff2/blob/master/src/woff2_compress.cc
    assert_eq!(input_size, 5_729_332);
    assert_eq!(output_size, 2_322_664);
    assert_eq!(
        calculate_hash(output).await?,
        "507421faf0310dae65c695f305b651379384f69a984dd04efdebdc999f96427a"
    );

    Ok(())
}

async fn calculate_hash<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let mut hasher = Sha256::new();
    hasher.update(read(&path).await?);
    let hash = hasher.finalize();
    let hex = hash.iter().fold(String::new(), |mut output, b| {
        output.push_str(&format!("{b:02x}"));
        output
    });
    Ok(hex)
}
