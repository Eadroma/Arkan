use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::PlatformRoute;

pub const RIOT_API_KEY_ENV: &str = "RIOT_API_KEY";
pub const DEFAULT_PLATFORM_ENV: &str = "ARKAN_DEFAULT_PLATFORM";
pub const DEFAULT_LANGUAGE_ENV: &str = "ARKAN_DEFAULT_LANGUAGE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    riot_api_key: Option<String>,
    default_platform: PlatformRoute,
    default_language: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_values(
            env::var(RIOT_API_KEY_ENV).ok(),
            env::var(DEFAULT_PLATFORM_ENV).ok(),
            env::var(DEFAULT_LANGUAGE_ENV).ok(),
        )
    }

    pub fn from_values(
        riot_api_key: Option<String>,
        default_platform: Option<String>,
        default_language: Option<String>,
    ) -> Result<Self, ConfigError> {
        let riot_api_key = riot_api_key.and_then(normalize_optional_value);

        let default_platform = match default_platform.and_then(normalize_optional_value) {
            Some(value) => value
                .parse::<PlatformRoute>()
                .map_err(|_| ConfigError::InvalidDefaultPlatform(value))?,
            None => PlatformRoute::Euw1,
        };

        let default_language = default_language
            .and_then(normalize_optional_value)
            .unwrap_or_else(|| "fr_FR".to_owned());

        Ok(Self {
            riot_api_key,
            default_platform,
            default_language,
        })
    }

    pub fn has_riot_api_key(&self) -> bool {
        self.riot_api_key.is_some()
    }

    pub fn riot_api_key(&self) -> Option<&str> {
        self.riot_api_key.as_deref()
    }

    pub fn masked_riot_api_key(&self) -> Option<String> {
        self.riot_api_key.as_deref().map(mask_secret)
    }

    pub fn default_platform(&self) -> PlatformRoute {
        self.default_platform
    }

    pub fn default_language(&self) -> &str {
        &self.default_language
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidDefaultPlatform(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDefaultPlatform(value) => {
                write!(f, "invalid default Riot platform route: {value}")
            }
        }
    }
}

impl Error for ConfigError {}

fn normalize_optional_value(value: String) -> Option<String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn mask_secret(secret: &str) -> String {
    let visible_suffix: String = secret
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if visible_suffix.is_empty() {
        "****".to_owned()
    } else {
        format!("****{visible_suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_euw_and_french_without_values() {
        let config = AppConfig::from_values(None, None, None).unwrap();

        assert!(!config.has_riot_api_key());
        assert_eq!(config.default_platform(), PlatformRoute::Euw1);
        assert_eq!(config.default_language(), "fr_FR");
    }

    #[test]
    fn trims_and_uses_configured_values() {
        let config = AppConfig::from_values(
            Some("  RGAPI-secret-value  ".to_owned()),
            Some(" na1 ".to_owned()),
            Some(" en_US ".to_owned()),
        )
        .unwrap();

        assert!(config.has_riot_api_key());
        assert_eq!(config.riot_api_key(), Some("RGAPI-secret-value"));
        assert_eq!(config.masked_riot_api_key(), Some("****alue".to_owned()));
        assert_eq!(config.default_platform(), PlatformRoute::Na1);
        assert_eq!(config.default_language(), "en_US");
    }

    #[test]
    fn rejects_invalid_platform() {
        let error = AppConfig::from_values(None, Some("EU".to_owned()), None).unwrap_err();

        assert_eq!(error, ConfigError::InvalidDefaultPlatform("EU".to_owned()));
    }
}
