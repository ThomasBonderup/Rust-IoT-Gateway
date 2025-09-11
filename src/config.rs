use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GatewayGfg {
    pub http: HttpCfg,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpCfg {
    pub bind: String,
}

pub fn load_config(config_file: Option<String>) -> anyhow::Result<GatewayGfg> {
    let mut builder = config::Config::builder()
        .add_source(config::File::with_name("gateway").required(false))
        .add_source(config::Environment::with_prefix("GATEWAY").separator("__"));
    if let Some(path) = config_file {
        builder = builder.add_source(config::File::with_name(&path));
    }
    Ok(builder.build()?.try_deserialize()?)
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

        let cfg = load_config(None).expect("confile file should load");

        assert_eq!(cfg.http.bind, "127.0.0.1:9999");

        env::set_current_dir(old_cwd).unwrap();
    }
}
