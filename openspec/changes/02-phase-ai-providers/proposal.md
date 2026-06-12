# Proposal: Phase 2 AI Providers

## Intent

Enable ScholarAI to interact with multiple AI providers (OpenAI, Anthropic, Ollama, etc.) through a unified LiteLLM integration. Currently the Python sidecar only exposes a `/health` endpoint. This change adds real AI capabilities while keeping API keys secure and the architecture clean.

## Scope

### In Scope
- LiteLLM integration in Python sidecar (chat completions + provider management)
- Secure API key storage via OS keychain (`keyring` crate)
- Rust bridge commands for provider CRUD and chat messaging
- Frontend ConfigPage for provider management (add, edit, test, activate)
- Support for 2 initial providers: OpenAI and local Ollama
- Shared Pydantic models for Rust-Python protocol validation

### Out of Scope
- Cost tracking and usage analytics (Phase 3)
- Automatic provider fallback / retry logic (Phase 3)
- Streaming chat responses (Phase 3)
- Advanced features: function calling, vision, embeddings

## Capabilities

### New Capabilities
- `ai-provider-management`: Add, edit, delete, test, and activate AI providers via UI
- `secure-api-key-storage`: Store provider API keys in OS credential store, metadata in JSON
- `chat-completion`: Send chat messages through configured providers via LiteLLM
- `python-sidecar-ai`: Python FastAPI routes for provider listing and chat completion

### Modified Capabilities
- None (this is a greenfield addition on top of Phase 1)

## Approach

1. **Python sidecar**: Add `litellm` to `pyproject.toml`, create `/api/v1/providers` and `/api/v1/chat/completions` endpoints with Pydantic models.
2. **Rust bridge**: Add `keyring` to `Cargo.toml`, extend `ai_bridge/mod.rs` with `post()` helper, add Tauri commands for provider CRUD and chat.
3. **Config storage**: Store provider metadata in JSON config; API keys in OS keychain keyed by provider ID.
4. **Frontend**: Build `ConfigPage` with provider form, test button, and activation toggle. Add `useAI` hook for chat requests.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `ai-service/main.py` | New | Provider routes and LiteLLM integration |
| `ai-service/pyproject.toml` | Modified | Add `litellm` dependency |
| `src-tauri/src/ai_bridge/mod.rs` | Modified | Extend HTTP helpers, add AI commands |
| `src-tauri/src/config/mod.rs` | Modified | Implement read/write + keyring integration |
| `src-tauri/src/commands/mod.rs` | Modified | Add provider management Tauri commands |
| `src-tauri/Cargo.toml` | Modified | Add `keyring` dependency |
| `src/pages/ConfigPage.tsx` | New | Provider settings UI |
| `src/` (hooks/stores) | New | `useAI` hook and provider state management |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| API keys stored in plain text | High | Enforce `keyring` — reject any plain-text storage |
| LiteLLM API changes | Med | Pin to stable version `>=1.0.0`, test before upgrade |
| Sidecar startup slowdown | Med | Monitor health-check timeout; lazy-load LiteLLM if needed |
| Linux keyring dependency | Med | Document `libsecret` requirement; graceful fallback to encrypted JSON |
| Provider error handling gaps | Med | Implement uniform error responses; surface to user in UI |
| No tests in codebase | High | Add at least provider-related unit tests in Python and Rust |

## Rollback Plan

1. Revert `ai-service` to Phase 1 state (remove LiteLLM routes).
2. Remove Rust Tauri commands for providers.
3. Restore `ConfigPage` to placeholder state.
4. Delete provider config JSON and clear keyring entries via OS credential manager.
5. Keep sidecar `/health` endpoint intact to maintain basic connectivity.

## Dependencies

- `litellm>=1.0.0` (Python)
- `keyring = "3"` (Rust)
- OS keychain support (macOS Keychain, Windows Credential Manager, Linux `libsecret`)

## Success Criteria

- [ ] Can add and save an OpenAI provider with API key stored in OS keychain
- [ ] Can add a local Ollama provider without API key
- [ ] Can test provider connection and get success/failure feedback
- [ ] Can send a chat message and receive a response through the configured provider
- [ ] API keys are never written to plain JSON or logs
- [ ] Provider list persists across app restarts
- [ ] Rust-Python communication uses validated Pydantic/serde models
