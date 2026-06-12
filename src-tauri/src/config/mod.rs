mod models;

use std::path::PathBuf;

use keyring::Entry;
use serde_json;
use thiserror::Error;

pub use models::{AppConfig, ProviderMeta, ProviderType};

const KEYRING_SERVICE: &str = "scholar-ai";

/// Top-level error type for config and keyring operations.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("KeyringUnavailable: install libsecret-1-0")]
    KeyringUnavailable(String),

    #[error("Keyring operation failed: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
}

/// Manages persistent configuration (config.json) and API keys (OS keychain).
///
/// **Design**: hybrid persistence
/// - Provider metadata lives in `config.json` (no `api_key` field — ever).
/// - API keys are stored in the OS keychain under service "scholar-ai".
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new `ConfigManager` pointing at the given config file path.
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path: config_path,
        }
    }

    // -------------------------------------------------------------------------
    // Config file I/O
    // -------------------------------------------------------------------------

    fn read_config_inner(&self) -> Result<AppConfig, ConfigError> {
        if !self.config_path.exists() {
            return Ok(AppConfig::default());
        }
        let contents = std::fs::read_to_string(&self.config_path)?;
        let config: AppConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    fn write_config_inner(&self, config: &AppConfig) -> Result<(), ConfigError> {
        let contents = serde_json::to_string_pretty(config)?;
        // Ensure the parent directory exists.
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.config_path, contents)?;
        Ok(())
    }

    /// Reads the current `AppConfig` from disk.
    pub fn read_config(&self) -> Result<AppConfig, ConfigError> {
        self.read_config_inner()
    }

    /// Writes the supplied `AppConfig` to disk.
    pub fn write_config(&self, config: &AppConfig) -> Result<(), ConfigError> {
        self.write_config_inner(config)
    }

    // -------------------------------------------------------------------------
    // Keyring helpers
    // -------------------------------------------------------------------------

    /// Returns a keyring `Entry` for the given provider ID under service
    /// `"scholar-ai"`.  If the OS keyring backend is unavailable on Linux
    /// (libsecret-1-0 not installed), this returns a clear error.
    fn entry_for(&self, id: &str) -> Result<Entry, ConfigError> {
        Entry::new(KEYRING_SERVICE, id).map_err(|e| match e {
            keyring::Error::PlatformFailure(ref inner)
                if cfg!(target_os = "linux") =>
            {
                ConfigError::KeyringUnavailable(format!(
                    "KeyringUnavailable: install libsecret-1-0 ({})",
                    inner
                ))
            }
            _ => ConfigError::KeyringUnavailable(format!(
                "KeyringUnavailable: {}",
                e
            )),
        })
    }

    /// Stores an API key in the OS keychain.
    ///
    /// - `id`    – provider identifier (used as the keyring `target`)
    /// - `key`   – the API key to store
    ///
    /// The key is **never** written to `config.json`.
    pub fn save_api_key(&self, id: &str, key: &str) -> Result<(), ConfigError> {
        let entry = self.entry_for(id)?;
        entry.set_password(key)?;
        Ok(())
    }

    /// Retrieves an API key from the OS keychain.
    ///
    /// Returns `Ok(None)` when the key does not exist yet.
    /// The key is **never** logged or written to disk.
    pub fn get_api_key(&self, id: &str) -> Result<Option<String>, ConfigError> {
        let entry = match self.entry_for(id) {
            Ok(e) => e,
            Err(ConfigError::KeyringUnavailable(_)) => return Ok(None),
            Err(e) => return Err(e),
        };
        match entry.get_password() {
            Ok(k) => Ok(Some(k)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

/// Deletes an API key from the OS keychain.
    ///
    /// Silently succeeds if the key does not exist.
    pub fn delete_api_key(&self, id: &str) -> Result<(), ConfigError> {
        // NOTE: keyring v3 dropped `delete_password`, so we call the
        // platform-native credential manager directly.
        let result = if cfg!(target_os = "macos") {
            std::process::Command::new("security")
                .args(["delete-generic-password", "-s", KEYRING_SERVICE, "-a", id])
                .output()
        } else if cfg!(target_os = "windows") {
            std::process::Command::new("cmdkey")
                .args(["/delete:", &format!("{}:{}", KEYRING_SERVICE, id)])
                .output()
        } else {
            // Linux / other: try secret-tool (from libsecret-1-0)
            std::process::Command::new("secret-tool")
                .args(["clear", "service", KEYRING_SERVICE, "account", id])
                .output()
        };

        match result {
            Ok(out) if out.status.success() => Ok(()),
            // "The specified item could not be found" is normal on Windows.
            Ok(_) if cfg!(target_os = "windows") => Ok(()),
            // On macOS, security returns 256 if the item doesn't exist — silently ok.
            Ok(out)
                if cfg!(target_os = "macos")
                    && (out.status.code() == Some(255) || out.status.code() == Some(256)) =>
            {
                Ok(())
            }
            // Linux secret-tool returns 1 if no matching secret found — silently ok.
            Ok(out) if cfg!(target_os = "linux") && !out.status.success() => Ok(()),
            Err(e) => Err(ConfigError::Io(e)),
            _ => Err(ConfigError::KeyringError(keyring::Error::PlatformFailure(
                format!("failed to delete keyring entry for {}", id).into(),
            ))),
        }
    }

    // -------------------------------------------------------------------------
    // Provider convenience methods
    // -------------------------------------------------------------------------

    /// Upserts a provider's metadata in config.json and stores its API key
    /// in the OS keychain (when `api_key` is `Some`).
    ///
    /// The `api_key` value is **never** written to the config file.
    pub fn save_provider(
        &self,
        meta: &ProviderMeta,
        api_key: Option<&str>,
    ) -> Result<(), ConfigError> {
        let mut config = self.read_config_inner()?;

        // Remove old entry for the same id (upsert).
        config.providers.retain(|p| p.id != meta.id);
        config.providers.push(meta.clone());

        self.write_config_inner(&config)?;

        if let Some(key) = api_key {
            // Derive a stable keyring ID that does NOT leak the key.
            let key_id = format!("provider-{}", meta.id);
            self.save_api_key(&key_id, key)?;
        }

        Ok(())
    }

    /// Loads a provider's metadata from config.json and retrieves its API key
    /// from the keychain.  The returned `ProviderMeta` is the on-disk copy
    /// (no `api_key` field).  The key is returned separately.
    pub fn load_provider(
        &self,
        id: &str,
    ) -> Result<(ProviderMeta, Option<String>), ConfigError> {
        let config = self.read_config_inner()?;
        let meta = config
            .providers
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| ConfigError::ProviderNotFound(id.to_string()))?;

        let key_id = format!("provider-{}", id);
        let key = self.get_api_key(&key_id)?;
        // Key is never logged — caller decides whether to use it.
        Ok((meta, key))
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use uuid::Uuid;

    use super::*;

    /// Helper: create a ConfigManager backed by a temp file.
    fn temp_config_manager() -> ConfigManager {
        let mut path = temp_dir();
        path.push(format!("scholar-ai-test-{}.json", Uuid::new_v4()));
        ConfigManager::new(path)
    }

    #[test]
    fn empty_config_roundtrip() {
        let mgr = temp_config_manager();
        let original = AppConfig::default();
        mgr.write_config(&original).unwrap();

        let loaded = mgr.read_config().unwrap();
        assert_eq!(loaded.providers.len(), 0);
        assert!(loaded.default_provider.is_none());
    }

    #[test]
    fn write_and_read_providers() {
        let mgr = temp_config_manager();

        let meta1 = ProviderMeta {
            id: "p1".into(),
            name: "OpenAI".into(),
            provider_type: ProviderType::OpenAi,
            base_url: None,
            model: Some("gpt-4o".into()),
            key_id: Some("provider-p1".into()),
            extra_headers: None,
        };

        let meta2 = ProviderMeta {
            id: "p2".into(),
            name: "Ollama".into(),
            provider_type: ProviderType::Ollama,
            base_url: Some("http://localhost:11434".into()),
            model: Some("llama3".into()),
            key_id: Some("provider-p2".into()),
            extra_headers: None,
        };

        let mut config = AppConfig::default();
        config.providers.push(meta1.clone());
        config.providers.push(meta2.clone());

        mgr.write_config(&config).unwrap();
        let loaded = mgr.read_config().unwrap();

        assert_eq!(loaded.providers.len(), 2);
        assert_eq!(loaded.providers[0].id, "p1");
        assert_eq!(loaded.providers[0].name, "OpenAI");
        assert_eq!(loaded.providers[0].provider_type, ProviderType::OpenAi);
        assert_eq!(loaded.providers[0].model.as_deref(), Some("gpt-4o"));
        assert!(loaded.providers[0].key_id.is_some());

        assert_eq!(loaded.providers[1].id, "p2");
        assert_eq!(loaded.providers[1].provider_type, ProviderType::Ollama);
        assert_eq!(
            loaded.providers[1].base_url.as_deref(),
            Some("http://localhost:11434")
        );
    }

    #[test]
    fn upsert_replaces_existing_provider() {
        let mgr = temp_config_manager();

        let meta_v1 = ProviderMeta {
            id: "p1".into(),
            name: "OpenAI v1".into(),
            provider_type: ProviderType::OpenAi,
            base_url: None,
            model: Some("gpt-4".into()),
            key_id: Some("provider-p1".into()),
            extra_headers: None,
        };

        let meta_v2 = ProviderMeta {
            id: "p1".into(),
            name: "OpenAI v2".into(),
            provider_type: ProviderType::OpenAi,
            base_url: Some("https://api.openai.com/v1".into()),
            model: Some("gpt-4o".into()),
            key_id: Some("provider-p1".into()),
            extra_headers: None,
        };

        let mut config = AppConfig::default();
        config.providers.push(meta_v1);
        mgr.write_config(&config).unwrap();

        let mut config2 = AppConfig::default();
        config2.providers.push(meta_v2.clone());
        mgr.write_config(&config2).unwrap();

        let loaded = mgr.read_config().unwrap();
        assert_eq!(loaded.providers.len(), 1);
        assert_eq!(loaded.providers[0].name, "OpenAI v2");
        assert_eq!(
            loaded.providers[0].base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
    }

    #[test]
    fn default_provider_persists() {
        let mgr = temp_config_manager();

        let mut config = AppConfig {
            default_provider: Some("p1".into()),
            providers: vec![ProviderMeta {
                id: "p1".into(),
                name: "Test".into(),
                provider_type: ProviderType::OpenAi,
                base_url: None,
                model: Some("gpt-4o".into()),
                key_id: Some("provider-p1".into()),
                extra_headers: None,
            }],
            theme: Some("dark".into()),
            locale: Some("en-US".into()),
        };

        mgr.write_config(&config).unwrap();
        let loaded = mgr.read_config().unwrap();

        assert_eq!(loaded.default_provider.as_deref(), Some("p1"));
        assert_eq!(loaded.theme.as_deref(), Some("dark"));
        assert_eq!(loaded.locale.as_deref(), Some("en-US"));
    }

    #[test]
    fn config_json_has_no_api_key_field() {
        let mgr = temp_config_manager();

        let meta = ProviderMeta {
            id: "p1".into(),
            name: "Test".into(),
            provider_type: ProviderType::OpenAi,
            base_url: None,
            model: Some("gpt-4o".into()),
            key_id: Some("provider-p1".into()),
            extra_headers: None,
        };

        let mut config = AppConfig::default();
        config.providers.push(meta);
        mgr.write_config(&config).unwrap();

        let raw = std::fs::read_to_string(&mgr.config_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let providers = json.get("providers").unwrap().as_array().unwrap();
        for p in providers {
            assert!(
                p.get("api_key").is_none(),
                "api_key must NEVER appear in config.json"
            );
        }
    }
}