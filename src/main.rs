mod config;
mod http;

use crate::config::{GatewayGfg, load_config};
use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    config: Option<String>,

    #[arg(long)]
    http_bind: Option<String>,

    #[arg(long)]
    print_bind: bool,
}

fn merge_overrides(mut cfg: GatewayGfg, cli: &Cli) -> GatewayGfg {
    cfg.http.bind = cli.http_bind.clone().unwrap_or(cfg.http.bind);
    cfg
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut cfg = load_config(cli.config.clone())?;

    cfg = merge_overrides(cfg, &cli);

    if cli.print_bind {
        println!("{}", cfg.http.bind);
        return Ok(());
    }

    println!("Loaded config: {:?}", cfg);

    http::serve(&cfg.http.bind).await?;
    Ok(())
}
