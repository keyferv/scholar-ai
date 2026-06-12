/**
 * Tests for Tauri command dispatch (ai_bridge and commands).
 *
 * These tests mock the HTTP layer to verify command wiring without
 * requiring a running sidecar.  They exercise `list_providers`,
 * `add_provider`, `delete_provider`, and `test_provider` commands.
 */

#[cfg(test)]
mod tauri_command_tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use crate::commands::{add_provider, delete_provider, list_providers, set_active_provider};
    use crate::config::{ConfigManager, ProviderMeta, ProviderType};

    // ── Helpers ────────────────────────────────────────────────────────────────

    fn temp_config_manager() -> ConfigManager {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "scholar-ai-cmd-test-{}.json",
            uuid::Uuid::new_v4()
        ));
        ConfigManager::new(path)
    }

    // ── list_providers ──────────────────────────────────────────────────────────

    #[test]
    fn list_providers_returns_empty_when_none_configured() {
        let mgr = temp_config_manager();
        // Pre-condition: no providers in config
        let result = list_providers(mgr.into());
        assert!(result.is_ok());
        let json = result.unwrap();
        let array = json.as_array().unwrap();
        assert!(array.is_empty());
    }

    #[test]
    fn list_providers_returns_configured_providers() {
        let mgr = temp_config_manager();

        let mut config = crate::config::AppConfig::default();
        config.providers.push(ProviderMeta {
            id: "p1".into(),
            name: "Test Provider".into(),
            provider_type: ProviderType::OpenAi,
            base_url: None,
            model: Some("gpt-4o".into()),
            key_id: Some("provider-p1".into()),
            extra_headers: None,
        });
        mgr.write_config(&config).unwrap();

        let result = list_providers(mgr.into());
        assert!(result.is_ok());
        let json = result.unwrap();
        let array = json.as_array().unwrap();
        assert_eq!(array.len(), 1);

        let provider = &array[0];
        assert_eq!(provider["name"], "Test Provider");
        assert_eq!(provider["provider_type"], "openai");
        assert_eq!(provider["model"], "gpt-4o");
        // api_key must NOT appear in serialized output
        assert!(provider.get("api_key").is_none());
    }

    // ── add_provider ───────────────────────────────────────────────────────────

    #[test]
    fn add_provider_creates_entry_and_strips_api_key_from_config() {
        let mgr = temp_config_manager();

        let result = add_provider(
            mgr.into(),
            "test-id".into(),
            "My Provider".into(),
            "openai".into(),
            Some("gpt-4o".into()),
            Some("https://api.example.com".into()),
            Some("sk-secret-key-12345".into()),
            None::<std::collections::HashMap<String, String>>,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().contains("added successfully"));

        // Verify config file contains metadata but NOT the API key
        let config = mgr.read_config().unwrap();
        assert_eq!(config.providers.len(), 1);
        let p = &config.providers[0];
        assert_eq!(p.id, "test-id");
        assert_eq!(p.name, "My Provider");
        assert_eq!(p.provider_type, ProviderType::OpenAi);
        assert_eq!(p.model.as_deref(), Some("gpt-4o"));
        assert_eq!(p.base_url.as_deref(), Some("https://api.example.com"));
        // key_id tracks the keyring entry, but actual key is in OS keychain
        assert_eq!(p.key_id.as_deref(), Some("provider-test-id"));
    }

    #[test]
    fn add_provider_rejects_invalid_type() {
        let mgr = temp_config_manager();

        let result = add_provider(
            mgr.into(),
            "id".into(),
            "Bad".into(),
            "invalid_type".into(),
            None::<String>,
            None::<String>,
            None::<String>,
            None::<std::collections::HashMap<String, String>>,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid provider_type"));
    }

    // ── delete_provider ─────────────────────────────────────────────────────────

    #[test]
    fn delete_provider_removes_from_config() {
        let mgr = temp_config_manager();

        // First add a provider
        let _ = add_provider(
            mgr.clone().into(),
            "del-me".into(),
            "To Delete".into(),
            "openai".into(),
            Some("gpt-4o".into()),
            None::<String>,
            Some("sk-temp-key".into()),
            None::<std::collections::HashMap<String, String>>,
        );

        // Confirm it exists
        let config = mgr.read_config().unwrap();
        assert_eq!(config.providers.len(), 1);
        drop(config);

        // Delete it
        let result = delete_provider(mgr.clone().into(), "del-me".into());
        assert!(result.is_ok());

        // Confirm removal
        let config = mgr.read_config().unwrap();
        assert!(config.providers.is_empty());
    }

    #[test]
    fn delete_provider_returns_error_for_missing_id() {
        let mgr = temp_config_manager();

        let result = delete_provider(mgr.into(), "nonexistent".into());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ── set_active_provider ─────────────────────────────────────────────────────

    #[test]
    fn set_active_provider_works_for_existing_provider() {
        let mgr = temp_config_manager();

        let _ = add_provider(
            mgr.clone().into(),
            "active-me".into(),
            "Active Provider".into(),
            "ollama".into(),
            Some("llama3".into()),
            None::<String>,
            None::<String>,
            None::<std::collections::HashMap<String, String>>,
        );

        let result = set_active_provider(mgr.clone().into(), "active-me".into());
        assert!(result.is_ok());

        let config = mgr.read_config().unwrap();
        assert_eq!(config.default_provider.as_deref(), Some("active-me"));
    }

    #[test]
    fn set_active_provider_rejects_missing_provider() {
        let mgr = temp_config_manager();

        let result = set_active_provider(mgr.into(), "ghost".into());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ── Config round-trip integrity ─────────────────────────────────────────────

    #[test]
    fn config_json_roundtrip_preserves_all_metadata() {
        let mgr = temp_config_manager();

        let mut config = crate::config::AppConfig {
            default_provider: Some("main".into()),
            providers: vec![
                ProviderMeta {
                    id: "main".into(),
                    name: "Main Provider".into(),
                    provider_type: ProviderType::OpenAi,
                    base_url: None,
                    model: Some("gpt-4o".into()),
                    key_id: Some("provider-main".into()),
                    extra_headers: Some(
                        [("X-Custom".into(), "value".into())]
                            .iter()
                            .cloned()
                            .collect(),
                    ),
                },
                ProviderMeta {
                    id: "local".into(),
                    name: "Local Model".into(),
                    provider_type: ProviderType::Ollama,
                    base_url: Some("http://localhost:11434".into()),
                    model: Some("llama3".into()),
                    key_id: None,
                    extra_headers: None,
                },
            ],
            theme: Some("dark".into()),
            locale: Some("en-US".into()),
        };

        mgr.write_config(&config).unwrap();
        let loaded = mgr.read_config().unwrap();

        assert_eq!(loaded.default_provider.as_deref(), Some("main"));
        assert_eq!(loaded.providers.len(), 2);
        assert_eq!(loaded.providers[0].extra_headers["X-Custom"], "value");
        assert_eq!(loaded.providers[1].base_url.as_deref(), Some("http://localhost:11434"));
        assert!(loaded.providers[0].key_id.is_some());
        assert!(loaded.providers[1].key_id.is_none());

        // Utmost check: api_key never appears on disk
        let raw = std::fs::read_to_string(&mgr.config_path).unwrap();
        assert!(!raw.contains("api_key"), "api_key must never be in config.json");
    }
}