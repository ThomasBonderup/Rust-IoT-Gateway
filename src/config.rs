use serde::Deserialize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

#[derive(Debug, Deserialize, Clone)]
pub struct GatewayGfg {
    #[serde(default)]
    pub http: HttpCfg,
    #[serde(default)]
    pub mqtt: MqttCfg,
    #[serde(default)]
    pub storage: StorageCfg,
    #[serde(default)]
    pub health: HealthCfg,
    #[serde(default)]
    pub ingest: IngestCfg,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpCfg {
    #[serde(default = "default_bind")]
    pub bind: SocketAddr,
}
impl Default for HttpCfg {
    fn default() -> Self {
        Self {
            bind: default_bind(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct MqttCfg {
    pub host: String,
    pub port: u16,
    pub client_id: String,
}
impl Default for MqttCfg {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 1883,
            client_id: "gw-1".into(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct StorageCfg {
    pub db_path: PathBuf,
    pub min_free_bytes: u64,
}
impl Default for StorageCfg {
    fn default() -> Self {
        Self {
            db_path: "./data.db".into(),
            min_free_bytes: 1,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct HealthCfg {
    pub require_mqtt: bool,
    pub require_disk: bool,
    pub probe_interval_ms: Option<u64>,
}
impl Default for HealthCfg {
    fn default() -> Self {
        Self {
            require_mqtt: (false),
            require_disk: (false),
            probe_interval_ms: Some(1000),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct IngestCfg {
    pub max_payload_bytes: usize,
    pub queue_capacity: usize,
    pub ack_mode: AckMode,
    pub require_auth: bool,
}
impl Default for IngestCfg {
    fn default() -> Self {
        Self {
            max_payload_bytes: 65536,
            queue_capacity: 10000,
            ack_mode: AckMode::Enqueue,
            require_auth: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AckMode {
    Enqueue,
    Sink,
}
impl Default for AckMode {
    fn default() -> Self {
        AckMode::Enqueue
    }
}

fn default_bind() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080)
}

impl GatewayGfg {
    pub fn load(path: Option<String>) -> anyhow::Result<Self> {
        Self::from_builder(build_config(path)?)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.mqtt.host.is_empty(), "mqtt.host cannot be empty");
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
