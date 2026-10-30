//! Settings configuration
//!
//! Manages user-configurable settings for the IME.
//! Default values are defined in `config/default.toml`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Default configuration TOML embedded from config/default.toml
const DEFAULT_CONFIG_TOML: &str = include_str!("../../config/default.toml");

/// Environment variable overriding the afrim configuration file path
pub const AFRIM_CONFIG_ENV: &str = "AFRIM_CONFIG";

/// Afrim engine settings
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AfrimSettings {
    /// Path to the afrim configuration file (optional).
    /// Relative paths are resolved against the afrim-desktop config directory.
    #[serde(default)]
    pub config_path: Option<String>,
}

/// Configuration settings for the IME
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Afrim engine settings
    #[serde(default)]
    pub afrim: AfrimSettings,
}

/// Recursively merge `overlay` TOML values on top of `base`.
fn merge_toml(base: &mut toml::Value, overlay: &toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(base_table), toml::Value::Table(overlay_table)) => {
            for (key, value) in overlay_table {
                if let Some(base_value) = base_table.get_mut(key) {
                    merge_toml(base_value, value);
                } else {
                    base_table.insert(key.clone(), value.clone());
                }
            }
        }
        (base, _) => {
            *base = overlay.clone();
        }
    }
}

/// Parse user TOML content merged on top of default.toml.
fn parse_with_defaults(user_content: &str) -> Result<Settings> {
    let mut base: toml::Value = toml::from_str(DEFAULT_CONFIG_TOML)?;
    let user: toml::Value = toml::from_str(user_content)?;
    merge_toml(&mut base, &user);
    let settings: Settings = base.try_into()?;
    Ok(settings)
}

/// Get the project directories for afrim-desktop-im.
fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("org", "fodydev", "afrim-desktop")
}

impl Settings {
    /// Get the data directory path
    pub fn data_dir() -> Option<PathBuf> {
        project_dirs().map(|dirs| dirs.data_dir().to_path_buf())
    }

    /// Get the configuration directory path
    pub fn config_dir() -> Option<PathBuf> {
        project_dirs().map(|dirs| dirs.config_dir().to_path_buf())
    }

    /// Get the configuration file path
    pub fn config_file() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.toml"))
    }

    /// Resolve the afrim configuration file to load.
    ///
    /// Order: `$AFRIM_CONFIG`, `[afrim] config_path`, then `afrim/config.toml`
    /// in the config directory. Returns `None` when none of them is set or
    /// exists, in which case the built-in sample dataset should be used.
    pub fn afrim_config_path(&self) -> Option<PathBuf> {
        if let Ok(env) = std::env::var(AFRIM_CONFIG_ENV)
            && !env.is_empty()
        {
            return Some(PathBuf::from(env));
        }

        let config_dir = Self::config_dir();

        if let Some(path) = self.afrim.config_path.as_deref().filter(|p| !p.is_empty()) {
            let path = PathBuf::from(path);
            return Some(match &config_dir {
                Some(dir) if path.is_relative() => dir.join(path),
                _ => path,
            });
        }

        config_dir
            .map(|dir| dir.join("afrim").join("config.toml"))
            .filter(|path| path.exists())
    }

    /// Load settings from the default configuration file.
    /// Falls back to embedded default.toml if the config file does not exist.
    pub fn load() -> Result<Self> {
        let Some(config_file) = Self::config_file() else {
            warn!("Could not determine config directory, using defaults");
            return Ok(Self::default());
        };

        if !config_file.exists() {
            debug!("Config file not found at {:?}, using defaults", config_file);
            return Ok(Self::default());
        }

        debug!("Loading config from {:?}", config_file);
        let content = fs::read_to_string(&config_file)?;
        parse_with_defaults(&content)
    }

    /// Ensure a default config file exists so users can discover and edit it.
    ///
    /// If the config file doesn't exist, creates the config directory and
    /// writes the embedded default configuration. Returns the config file path.
    pub fn ensure_default_config() -> Option<PathBuf> {
        let config_file = Self::config_file()?;

        if config_file.exists() {
            return Some(config_file);
        }

        if let Some(parent) = config_file.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            warn!("Failed to create config directory {:?}: {}", parent, e);
            return Some(config_file);
        }

        match fs::write(&config_file, DEFAULT_CONFIG_TOML) {
            Ok(()) => {
                debug!("Created default config at {:?}", config_file);
            }
            Err(e) => {
                warn!("Failed to write default config to {:?}: {}", config_file, e);
            }
        }

        Some(config_file)
    }

    /// Load settings from a specific file, merged on top of defaults.
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        parse_with_defaults(&content)
    }

    /// Save settings to the default configuration file
    pub fn save(&self) -> Result<()> {
        let Some(config_file) = Self::config_file() else {
            anyhow::bail!("Could not determine config directory");
        };

        // Create config directory if it doesn't exist
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)?;
        }

        debug!("Saving config to {:?}", config_file);
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_file, content)?;
        Ok(())
    }

    /// Save settings to a specific file
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.afrim.config_path, None);
        // The embedded default.toml must parse to the same defaults
        let parsed = parse_with_defaults("").unwrap();
        assert_eq!(parsed.afrim.config_path, None);
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut settings = Settings::default();
        settings.afrim.config_path = Some("/tmp/afrim.toml".into());
        let toml_str = toml::to_string(&settings).unwrap();
        let loaded: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.afrim.config_path, settings.afrim.config_path);
    }

    #[test]
    fn test_load_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[afrim]
config_path = "/data/afrim/bamileke.toml"
"#
        )
        .unwrap();

        let settings = Settings::load_from(file.path()).unwrap();
        assert_eq!(
            settings.afrim.config_path.as_deref(),
            Some("/data/afrim/bamileke.toml")
        );
    }

    #[test]
    fn test_unknown_legacy_sections_are_ignored() {
        // Configs written by the former Japanese engine must still load
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[conversion]
num_candidates = 5

[learning]
enabled = true
"#
        )
        .unwrap();

        let settings = Settings::load_from(file.path()).unwrap();
        assert_eq!(settings.afrim.config_path, None);
    }

    #[test]
    fn test_absolute_config_path() {
        let settings = Settings {
            afrim: AfrimSettings {
                config_path: Some("/abs/config.toml".into()),
            },
        };
        // Only meaningful when AFRIM_CONFIG is not set in the test environment
        if std::env::var(AFRIM_CONFIG_ENV).is_err() {
            assert_eq!(
                settings.afrim_config_path(),
                Some(PathBuf::from("/abs/config.toml"))
            );
        }
    }
}
