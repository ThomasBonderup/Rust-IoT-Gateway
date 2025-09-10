mod config;
mod http;

use crate::config::{GatewayGfg, load_config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg: GatewayGfg = load_config(None)?;
    println!("Loaded config: {:?}", cfg);

    http::serve(&cfg.http.bind).await?;
    Ok(())
}
