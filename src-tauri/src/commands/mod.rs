use serde_json::json;

use crate::config::{ConfigManager, ProviderMeta, ProviderType};

/// Tauri command: basic health check.
#[tauri::command]
pub fn health_check() -> serde_json::Value {
    json!({
        "name": "ScholarAI",
        "version": "0.1.0",
        "status": "running",
    })
}

/// Tauri command: get application info.
#[tauri::command]
pub fn get_app_info() -> serde_json::Value {
    json!({
        "name": "ScholarAI",
        "version": "0.1.0",
    })
}

/// Tauri command: list all configured AI providers.
#[tauri::command]
pub fn list_providers(config: tauri::State<'_, ConfigManager>) -> Result<serde_json::Value, String> {
    let config = config
        .read_config()
        .map_err(|e| format!("Failed to read config: {}", e))?;
    Ok(json!(config.providers))
}

/// Tauri command: add a new AI provider.
#[tauri::command]
pub fn add_provider(
    config: tauri::State<'_, ConfigManager>,
    id: String,
    name: String,
    provider_type: String,
    model: Option<String>,
    base_url: Option<String>,
    api_key: Option<String>,
    extra_headers: Option<std::collections::HashMap<String, String>>,
) -> Result<String, String> {
    let ptype = match provider_type.as_str() {
        "openai" => ProviderType::OpenAi,
        "anthropic" => ProviderType::Anthropic,
        "ollama" => ProviderType::Ollama,
        "openrouter" => ProviderType::OpenRouter,
        "custom" => ProviderType::Custom,
        _ => {
            return Err(format!(
                "Invalid provider_type '{}'. Must be one of: openai, anthropic, ollama, openrouter, custom",
                provider_type
            ));
        }
    };

    let meta = ProviderMeta {
        id: id.clone(),
        name,
        provider_type: ptype,
        base_url,
        model,
        key_id: api_key.as_ref().map(|_| format!("provider-{}", id)),
        extra_headers,
    };

    config
        .save_provider(&meta, api_key.as_deref())
        .map_err(|e| format!("Failed to save provider: {}", e))?;

    Ok(format!("Provider '{}' added successfully", id))
}

/// Tauri command: update an existing AI provider's metadata.
#[tauri::command]
pub fn update_provider(
    config: tauri::State<'_, ConfigManager>,
    id: String,
    name: Option<String>,
    provider_type: Option<String>,
    model: Option<Option<String>>,
    base_url: Option<Option<String>>,
    api_key: Option<String>,
    extra_headers: Option<Option<std::collections::HashMap<String, String>>>,
) -> Result<String, String> {
    // Load existing metadata first.
    let (mut meta, _key) = config
        .load_provider(&id)
        .map_err(|e| format!("Failed to load provider: {}", e))?;

    if let Some(name) = name {
        meta.name = name;
    }
    if let Some(ptype_str) = provider_type {
        meta.provider_type = match ptype_str.as_str() {
            "openai" => ProviderType::OpenAi,
            "anthropic" => ProviderType::Anthropic,
            "ollama" => ProviderType::Ollama,
            "openrouter" => ProviderType::OpenRouter,
            "custom" => ProviderType::Custom,
            _ => {
                return Err(format!(
                    "Invalid provider_type '{}'. Must be one of: openai, anthropic, ollama, openrouter, custom",
                    ptype_str
                ));
            }
        };
    }
    if let Some(model) = model {
        meta.model = model;
    }
    if let Some(base_url) = base_url {
        meta.base_url = base_url;
    }
    if let Some(headers) = extra_headers {
        meta.extra_headers = headers;
    }

    config
        .save_provider(&meta, api_key.as_deref())
        .map_err(|e| format!("Failed to update provider: {}", e))?;

    Ok(format!("Provider '{}' updated successfully", id))
}

/// Tauri command: delete an AI provider by ID.
///
/// Removes the provider's metadata from config.json AND deletes its
/// API key from the OS keychain.
#[tauri::command]
pub fn delete_provider(
    config: tauri::State<'_, ConfigManager>,
    id: String,
) -> Result<String, String> {
    // Remove from config.json.
    let mut config_data = config
        .read_config()
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let idx = config_data
        .providers
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| format!("Provider '{}' not found", id))?;

    config_data.providers.remove(idx);

    // If this was the default provider, clear the default.
    if config_data.default_provider.as_deref() == Some(&id) {
        config_data.default_provider = None;
    }

    config
        .write_config(&config_data)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    // Remove from keychain.
    let key_id = format!("provider-{}", id);
    config
        .delete_api_key(&key_id)
        .map_err(|e| format!("Failed to delete API key: {}", e))?;

    Ok(format!("Provider '{}' deleted successfully", id))
}

/// Tauri command: set the default/active AI provider.
#[tauri::command]
pub fn set_active_provider(
    config: tauri::State<'_, ConfigManager>,
    id: String,
) -> Result<String, String> {
    let mut config_data = config
        .read_config()
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // Verify the provider exists.
    if !config_data.providers.iter().any(|p| p.id == id) {
        return Err(format!("Provider '{}' not found", id));
    }

    config_data.default_provider = Some(id.clone());

    config
        .write_config(&config_data)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(format!("Active provider set to '{}'", id))
}