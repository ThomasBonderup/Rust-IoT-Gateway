mod http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    http::serve("0.0.0.0:8080").await?;
    Ok(())
}
