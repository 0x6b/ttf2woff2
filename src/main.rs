use ttf2woff2::Converter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    Converter::try_new().await?.write_to_woff2().await
}
