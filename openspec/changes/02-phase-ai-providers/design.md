# Design: Phase 2 AI Providers

## Technical Approach

Two-layer architecture: Python sidecar owns all AI logic via LiteLLM (single source of truth). Rust bridge acts as thin HTTP proxy + secure config manager (OS keychain + JSON metadata). Frontend provides config UI and chat interface via Tauri IPC commands.

## Architecture Decisions

| Decision | Option A | Option B | Option C | Choice | Rationale |
|----------|----------|----------|----------|--------|-----------|
| API Key Storage | Plain JSON | SQLite encrypted | OS keychain (`keyring`) | **OS keychain** | Best security/UX tradeoff; keys never touch disk unencrypted; no password UX |
| Provider Metadata | JSON file | SQLite table | TOML | **JSON file** | Readable, easy backup; no schema migration needed; separate from DB |
| LiteLLM Integration | Library (`litellm.completion`) | Proxy (`litellm --proxy`) | **Library** | Simpler, single process, no extra port; existing sidecar pattern works |
| AI Logic Ownership | Rust (call providers directly) | Python sidecar (Rust proxies) | **Python sidecar** | All AI logic in one place; Python ecosystem has better LLM tooling; Rust stays thin |
| Rust↔Python Protocol | Loose JSON | Shared Pydantic models + serde | **Pydantic + serde** | Type safety on both sides; JSON schema generated from Pydantic; validation at boundary |
| Chat Response | Non-streaming | Streaming (SSE) | **Non-streaming** | Phase 3 scope; reduces complexity; validate architecture first |

## Data Flow

### Provider CRUD Flow

```
ConfigPage ──invoke──▶ Tauri Command ──▶ ConfigManager ──write──▶ config.json (metadata)
                          │                                         │
                          ▼                                         ▼
                     keyring::set() ◀── API key ──── keyring::get() ▶── read
                          │
                     OS Credential Store
```

### Chat Completion Flow

```
Frontend ──invoke("send_chat_message")──▶ Tauri Command
                                                  │
                                                  ▼
                                        AiBridge::post("/api/v1/chat/completions")
                                                  │
                                                  ▼ HTTP POST
                                        Python Sidecar (FastAPI)
                                                  │
                                                  ▼
                                        litellm.completion(model=..., messages=...)
                                                  │
                                                  ▼
                                        OpenAI / Anthropic / Ollama API
```

### Sidecar Startup (unchanged)

```
Tauri::setup ──▶ AiBridge::new() ──▶ spawn_sidecar()
                                       │
                                       ▼
                                  python -m uvicorn main:app (port 8321)
                                       │
                                       ▼
                                  health poll loop (30 retries × 500ms)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `ai-service/pyproject.toml` | Modify | Add `litellm>=1.0.0` dependency |
| `ai-service/main.py` | Modify | Add router includes, lifespan startup (pre-load litellm) |
| `ai-service/routers/providers.py` | Create | `/api/v1/providers` CRUD + test endpoint |
| `ai-service/routers/chat.py` | Create | `/api/v1/chat/completions` endpoint |
| `ai-service/models/provider.py` | Create | Pydantic models: `ProviderConfig`, `ProviderTestResult`, `ChatRequest`, `ChatResponse` |
| `ai-service/services/provider_service.py` | Create | Business logic: list/add/update/delete/test providers via litellm |
| `ai-service/services/chat_service.py` | Create | Business logic: route chat to correct provider via litellm |
| `src-tauri/Cargo.toml` | Modify | Add `keyring = "3"` dependency |
| `src-tauri/src/config/mod.rs` | Modify | Implement `read_config`, `write_config`, keyring helpers |
| `src-tauri/src/config/models.rs` | Create | Serde structs: `AppConfig`, `ProviderMeta` |
| `src-tauri/src/ai_bridge/mod.rs` | Modify | Add `post()` helper, JSON body support |
| `src-tauri/src/commands/mod.rs` | Modify | Add 6 Tauri commands for provider CRUD + chat |
| `src-tauri/src/lib.rs` | Modify | Register new commands, pass ConfigManager to AiBridge |
| `src-tauri/capabilities/default.json` | Modify | Add permissions for new commands |
| `src/pages/ConfigPage.tsx` | Modify | Build provider management UI |
| `src/stores/providers.ts` | Create | Zustand store for provider state |
| `src/hooks/useAI.ts` | Create | Hook wrapping `send_chat_message` Tauri command |

## Interfaces / Contracts

### Python Pydantic Models (`models/provider.py`)

```python
class ProviderMeta(BaseModel):
    id: str
    name: str
    provider_type: Literal["openai", "anthropic", "ollama", "custom"]
    base_url: str | None = None
    model: str
    is_active: bool = False

class ProviderCreateRequest(BaseModel):
    name: str
    provider_type: Literal["openai", "anthropic", "ollama", "custom"]
    base_url: str | None = None
    model: str
    api_key: str | None = None

class ProviderTestResult(BaseModel):
    success: bool
    message: str
    latency_ms: int

class ChatMessage(BaseModel):
    role: Literal["system", "user", "assistant"]
    content: str

class ChatRequest(BaseModel):
    provider_id: str
    messages: list[ChatMessage]
    max_tokens: int = 1024
    temperature: float = 0.7

class ChatResponse(BaseModel):
    content: str
    model: str
    usage: dict
```

### Rust Serde Structs (`config/models.rs`)

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct ProviderMeta {
    pub id: String,
    pub name: String,
    pub provider_type: String,  // "openai" | "anthropic" | "ollama" | "custom"
    pub base_url: Option<String>,
    pub model: String,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub providers: Vec<ProviderMeta>,
}
```

### Tauri Commands (added to `commands/mod.rs`)

```rust
#[tauri::command] fn list_providers(state) -> Result<Vec<ProviderMeta>, String>
#[tauri::command] fn add_provider(state, name, provider_type, model, base_url, api_key) -> Result<ProviderMeta, String>
#[tauri::command] fn update_provider(state, id, name, provider_type, model, base_url, api_key) -> Result<ProviderMeta, String>
#[tauri::command] fn delete_provider(state, id) -> Result<(), String>
#[tauri::command] fn test_provider(state, id) -> Result<ProviderTestResult, String>
#[tauri::command] async fn send_chat_message(state, provider_id, messages) -> Result<ChatResponse, String>
```

### Python Endpoints

```
GET    /api/v1/providers              → list all providers (from Rust via JSON)
POST   /api/v1/providers/test         → test a provider (litellm ping)
POST   /api/v1/chat/completions       → chat completion (litellm.completion)
```

Note: Provider CRUD (add/update/delete) lives entirely in Rust. Python only receives `provider_id` + `api_key` at chat time. The `/providers/test` endpoint receives the full config + key for one-time validation.

## Error Handling Strategy

| Layer | Error Type | Handling |
|-------|-----------|----------|
| Python sidecar | LiteLLM error (rate limit, invalid key, timeout) | Return `{success: false, message: "<specific error>"}` with HTTP 4xx/5xx |
| Rust bridge | HTTP request to sidecar fails | Return `Err("Sidecar unavailable")` to frontend |
| Rust config | Keyring not available | Log warning; return `Err("Keyring unavailable — check OS credential store")` |
| Rust config | Config file corrupt | Return `Err("Config corrupted — reset recommended")` |
| Frontend | Tauri command returns Err | Show toast notification with error message |
| Frontend | Provider test fails | Show inline error in provider form |

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Python unit | Pydantic model validation | pytest: valid/invalid inputs for each model |
| Python unit | `provider_service` logic | Mock `litellm.completion`; verify routing by provider_type |
| Python integration | `/chat/completions` endpoint | FastAPI TestClient with mocked litellm |
| Rust unit | Config read/write round-trip | Write config → read → assert equality |
| Rust integration | Tauri command flow | Mock sidecar HTTP; verify command dispatch |
| Frontend | ConfigPage form rendering | Vitest + React Testing Library |

## Migration / Rollout

No migration required. This is greenfield on top of Phase 1. The `config.json` file starts empty (`{"providers": []}`) on first run.

Keyring entries are created on-demand. No bulk migration needed.

## Implementation Order

1. **Python models + endpoints** (foundation — no Rust dependency)
2. **Rust config + keyring** (secure storage layer)
3. **Rust commands + AiBridge extension** (wire commands to sidecar)
4. **Frontend store + ConfigPage UI** (user-facing)
5. **Error handling + polish** (surfacing errors properly)

## Open Questions

- [ ] Should the Python sidecar return provider metadata (names, models) or should Rust keep a static list of known provider types? **Recommendation: Rust keeps the enum, Python validates against litellm at test time.**
- [ ] Ollama base_url defaults — should we auto-detect `http://localhost:11434` or require explicit entry? **Recommendation: pre-fill default for Ollama type, editable.**
- [ ] How to handle keyring unavailability on Linux (libsecret/dbus missing)? **Recommendation: graceful fallback — log warning, store in encrypted JSON with user-provided password (Phase 3). For Phase 2, require libsecret and document it.**
