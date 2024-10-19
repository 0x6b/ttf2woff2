use anyhow::Result;
use camino::Utf8PathBuf;
use sha2_hasher::Sha2Hasher;
use ttf2woff2::{BrotliQuality, Converter};

#[tokio::test]
async fn test() -> Result<()> {
    let root = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let input = root.join("NotoSansJP-Medium.ttf");
    let output = root.join("NotoSansJP-Medium.woff2");

    let converter =
        Converter::from_file(input, Some(output.clone()), BrotliQuality::default()).await?;
    converter.write_to_woff2().await?;

    // pre-calculated SHA-256 hash and output file size using `woff2_compress` command from
    // https://github.com/google/woff2/blob/master/src/woff2_compress.cc
    assert_eq!(
        output.sha256().await?,
        "507421faf0310dae65c695f305b651379384f69a984dd04efdebdc999f96427a"
    );

    Ok(())
}
