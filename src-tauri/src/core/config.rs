use crate::core::error::OxideError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub adb_path: Option<String>,
    pub fastboot_path: Option<String>,
    pub theme: Option<String>,
    pub audit_enabled: bool,
}

impl AppConfig {
    pub fn data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("oxide"))
    }

    pub fn path() -> Option<PathBuf> {
        Self::data_dir().map(|d| d.join("config.json"))
    }

    pub fn load() -> Result<Self, OxideError> {
        let path = Self::path().ok_or_else(|| {
            OxideError::Config("cannot determine platform data directory".into())
        })?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let bytes = std::fs::read(&path).map_err(|e| {
            OxideError::Config(format!("cannot read config: {e}"))
        })?;

        serde_json::from_slice(&bytes).map_err(|e| {
            OxideError::Config(format!("invalid config JSON: {e}"))
        })
    }

    pub fn save(&self) -> Result<(), OxideError> {
        let dir = Self::data_dir().ok_or_else(|| {
            OxideError::ConfigSave("cannot determine platform data directory".into())
        })?;

        std::fs::create_dir_all(&dir).map_err(|e| {
            OxideError::ConfigSave(format!("cannot create config dir: {e}"))
        })?;

        let path = dir.join("config.json");
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| {
            OxideError::ConfigSave(format!("cannot serialize config: {e}"))
        })?;

        std::fs::write(&path, bytes).map_err(|e| {
            OxideError::ConfigSave(format!("cannot write config: {e}"))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AppConfig::default();
        assert!(cfg.adb_path.is_none());
        assert!(cfg.fastboot_path.is_none());
        assert!(cfg.theme.is_none());
        assert!(!cfg.audit_enabled);
    }

    #[test]
    fn test_roundtrip() {
        let cfg = AppConfig {
            adb_path: Some("/usr/bin/adb".into()),
            fastboot_path: None,
            theme: Some("dark".into()),
            audit_enabled: true,
        };

        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.adb_path, Some("/usr/bin/adb".into()));
        assert!(deserialized.fastboot_path.is_none());
        assert_eq!(deserialized.theme, Some("dark".into()));
        assert!(deserialized.audit_enabled);
    }

    #[test]
    fn test_data_dir_returns_something() {
        assert!(AppConfig::data_dir().is_some());
    }

    #[test]
    fn test_load_returns_default_when_no_file() {
        // doesn't exist yet, so returns default
        let cfg = AppConfig::load();
        assert!(cfg.is_ok());
    }
}
