use serde::Deserialize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

#[derive(Debug, Deserialize, Clone)]
pub struct GatewayGfg {
    pub http: HttpCfg,
    pub mqtt: MqttCfg,
    pub storage: StorageCfg,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpCfg {
    #[serde(default = "default_bind")]
    pub bind: SocketAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MqttCfg {
    pub host: String,
    pub port: u16,
    pub client_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageCfg {
    pub db_path: PathBuf,
    pub min_free_bytes: u64,
}

fn default_bind() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080)
}

impl GatewayGfg {
    pub fn load(path: Option<String>) -> anyhow::Result<Self> {
        Self::from_builder(build_config(path)?)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.http.bind.port() != 0, "http.bind port cannot be 0");
        Ok(())
    }

    fn from_builder(cfg: config::Config) -> anyhow::Result<Self> {
        Ok(cfg.try_deserialize()?)
    }
}

fn build_config(path: Option<String>) -> anyhow::Result<config::Config> {
    use config::{Config, Environment, File};
    let mut builder = Config::builder()
        .add_source(File::with_name("gateway").required(false))
        .add_source(Environment::with_prefix("GATEWAY").separator("__"));
    if let Some(path) = path {
        builder = builder.add_source(File::with_name(&path));
    }
    Ok(builder.build()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};
    use tempfile::tempdir;

    #[test]
    fn loads_from_config_file_in_cwd() {
        let dir = tempdir().expect("failed to create temp dir for test");
        let toml = r#"
            [http]
            bind = "127.0.0.1:9999"
        "#;
        fs::write(dir.path().join("gateway.toml"), toml).unwrap();

        let old_cwd = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let cfg = GatewayGfg::load(None).expect("confile file should load");

        assert_eq!(
            cfg.http.bind,
            "127.0.0.1:9999".parse::<SocketAddr>().unwrap()
        );

        env::set_current_dir(old_cwd).unwrap();
    }
}
