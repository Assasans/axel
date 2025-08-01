use std::net::SocketAddr;
use std::path::PathBuf;

use config::{Config, ConfigError};
use jwt_simple::prelude::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
#[allow(unused)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
  pub static_server: StaticServerSettings,
  pub api_server: ApiServerSettings,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StaticServerSettings {
  pub bind_address: SocketAddr,
  pub public_url: Url,
  pub resources_root: PathBuf,
  pub upstream_url: Option<Url>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ApiServerSettings {
  pub bind_address: SocketAddr,
  pub public_url: Url,
}

impl Settings {
  pub fn new() -> Result<Self, ConfigError> {
    let settings = Config::builder()
      .add_source(config::File::with_name("config/default"))
      .add_source(config::File::with_name("config/local").required(false))
      .build()?;

    settings.try_deserialize()
  }
}
