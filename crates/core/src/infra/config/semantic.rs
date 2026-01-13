use std::collections::HashSet;

use super::ConfigError;
use crate::domain::model::{
  AppConfig,
  CategoryConfig
};

pub fn validate_semantic(
  app: &AppConfig,
  categories: &[CategoryConfig]
) -> Result<(), ConfigError> {
  let mut category_domains =
    HashSet::new();

  for c in categories {
    for d in &c.domains {
      category_domains
        .insert(d.as_str());
    }
  }

  for domain in app.domains.keys() {
    if !category_domains
      .contains(domain.as_str())
    {
      return Err(ConfigError::Invalid(
        format!(
          "domain '{domain}' missing \
           from categories.toml"
        )
      ));
    }
  }

  Ok(())
}
