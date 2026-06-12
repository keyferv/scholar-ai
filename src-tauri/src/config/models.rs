use serde::{Deserialize, Serialize};

/// Metadata for an AI provider connection.
/// The api_key is NEVER stored here — it lives in the OS keychain only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderMeta {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    pub model: Option<String>,
    /// Keyring entry ID used to look up the API key in the OS keychain.
    /// Maps to the keyring `target` field under service "scholar-ai".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    OpenAi,
    Anthropic,
    Ollama,
    OpenRouter,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub providers: Vec<ProviderMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}