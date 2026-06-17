use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;

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
        let dot_env = read_local_dotenv();

        Self::from_values(
            env::var(RIOT_API_KEY_ENV)
                .ok()
                .or_else(|| dot_env_value(&dot_env, RIOT_API_KEY_ENV)),
            env::var(DEFAULT_PLATFORM_ENV)
                .ok()
                .or_else(|| dot_env_value(&dot_env, DEFAULT_PLATFORM_ENV)),
            env::var(DEFAULT_LANGUAGE_ENV)
                .ok()
                .or_else(|| dot_env_value(&dot_env, DEFAULT_LANGUAGE_ENV)),
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

fn read_local_dotenv() -> Option<String> {
    dotenv_candidates()
        .into_iter()
        .find_map(|path| fs::read_to_string(path).ok())
}

fn dotenv_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir.join(".env"));
    }

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let manifest_dir = PathBuf::from(manifest_dir);
        candidates.push(manifest_dir.join(".env"));

        if let Some(parent) = manifest_dir.parent() {
            candidates.push(parent.join(".env"));
        }
    }

    candidates
}

fn dot_env_value(dot_env: &Option<String>, key: &str) -> Option<String> {
    dot_env
        .as_deref()
        .and_then(|content| parse_dotenv_value(content, key))
}

fn parse_dotenv_value(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        let (candidate_key, value) = trimmed.split_once('=')?;

        if candidate_key.trim() == key {
            Some(unquote_dotenv_value(value.trim()))
        } else {
            None
        }
    })
}

fn unquote_dotenv_value(value: &str) -> String {
    if value.len() >= 2 {
        let mut chars = value.chars();
        let first = chars.next();
        let last = value.chars().last();

        if matches!(
            (first, last),
            (Some('"'), Some('"')) | (Some('\''), Some('\''))
        ) {
            return value[1..value.len() - 1].to_owned();
        }
    }

    value.to_owned()
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

    #[test]
    fn parses_dotenv_values() {
        let content = r#"
            # local development
            RIOT_API_KEY="RGAPI-local-key"
            ARKAN_DEFAULT_PLATFORM=EUW1
        "#;

        assert_eq!(
            parse_dotenv_value(content, RIOT_API_KEY_ENV),
            Some("RGAPI-local-key".to_owned())
        );
        assert_eq!(
            parse_dotenv_value(content, DEFAULT_PLATFORM_ENV),
            Some("EUW1".to_owned())
        );
        assert_eq!(parse_dotenv_value(content, DEFAULT_LANGUAGE_ENV), None);
    }
}
