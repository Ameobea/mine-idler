use foundations::{settings::settings, telemetry::settings::TelemetrySettings};
use serde_default_utils::*;

#[serde_inline_default]
#[settings]
pub struct ServerSettings {
  #[serde_inline_default(5900)]
  pub port: u16,
}

#[serde_inline_default]
#[settings]
pub struct DatabaseSettings {
  #[serde_inline_default("localhost".to_owned())]
  pub host: String,
  #[serde_inline_default(5432)]
  pub port: u16,
  #[serde_inline_default("mine_idler".to_owned())]
  pub username: String,
  pub password: String,
  #[serde_inline_default("mine_idler".to_owned())]
  pub database: String,
}

#[serde_inline_default]
#[settings]
pub struct AuthSettings {
  // 6 months
  #[serde_inline_default(60 * 60 * 24 * 30 * 6)]
  pub session_token_lifetime_seconds: u64,
}

#[settings]
pub struct Settings {
  /// Telemetry settings.
  pub telemetry: TelemetrySettings,

  pub server: ServerSettings,
  pub database: DatabaseSettings,
  pub auth: AuthSettings,
}
