//! Loads the TOML configuration bundle
//! (app/domains/feeds) and normalizes
//! it into `AppConfig` + feed list.

mod defaults;
mod error;
mod feeds;
mod loader;
mod parse;
mod paths;
mod raw;
mod schema;
mod semantic;

pub use error::ConfigError;
pub use loader::{
  ConfigLoader,
  LoadedConfig
};
pub use semantic::validate_semantic;
