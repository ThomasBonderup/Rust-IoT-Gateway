mod app;
mod config;
mod http;
mod ingest;
mod metrics;
mod readiness;

use crate::config::GatewayGfg;
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    config: Option<String>,

    #[arg(long)]
    http_bind: Option<SocketAddr>,

    #[arg(long)]
    print_bind: bool,
}

fn merge_overrides(mut cfg: GatewayGfg, cli: &Cli) -> GatewayGfg {
    cfg.http.bind = cli.http_bind.unwrap_or(cfg.http.bind);
    cfg
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .json()
        .with_target(true)
        .with_current_span(true)
        .with_line_number(true)
        .with_file(true)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut cfg = GatewayGfg::load(cli.config.clone())?;
    cfg = merge_overrides(cfg, &cli);
    cfg.validate()?;

    if cli.print_bind {
        println!("{}", cfg.http.bind);
        return Ok(());
    }

    println!("Loaded config: {:?}", cfg);

    init_tracing();
    let cfg = Arc::new(cfg);
    http::serve(cfg.http.bind, cfg).await?;
    Ok(())
}
