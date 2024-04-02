use log::info;
use ttf2woff2_rs::Converter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (before, after) = Converter::try_new().await?.to_woff2().await?;
    info!("{before} bytes → {after} bytes ({:.2}%)", 100.0 * (after as f64) / (before as f64));

    Ok(())
}
