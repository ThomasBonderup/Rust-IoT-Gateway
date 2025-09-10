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
