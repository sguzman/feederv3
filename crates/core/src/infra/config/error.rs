#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
  #[error("io: {0}")]
  Io(#[from] std::io::Error),
  #[error("toml: {0}")]
  Toml(#[from] toml::de::Error),
  #[error("invalid config: {0}")]
  Invalid(String)
}
