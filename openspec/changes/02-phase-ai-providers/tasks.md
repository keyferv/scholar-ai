# Tasks: Phase 2 — AI Providers

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~2000 (Python ~735, Rust ~655, Frontend ~580, Tests ~400) |
| 400-line budget risk | **High** |
| Chained PRs recommended | **Yes** |
| Suggested split | PR 1 → PR 2 → PR 3 → PR 4 |
| Delivery strategy | chained PRs (stacked to main) |
| Chain strategy | stacked-to-main |
| Decision needed before apply | No |

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Python sidecar LiteLLM integration (models + endpoints) | PR 1 | Base: main; standalone FastAPI service |
| 2 | Rust secure config (keyring + metadata models) | PR 2 | Base: main; independent ConfigManager extension |
| 3 | Rust bridge commands + AiBridge HTTP client | PR 3 | Base: main; depends on Unit 1+2 contracts |
| 4 | Frontend ConfigPage UI + providers store | PR 4 | Base: main; depends on Unit 3 commands |

---

## Phase 1: Python Sidecar LiteLLM Integration (T1)

- [ ] 1.1 Add `litellm>=1.0.0,<2.0.0` to `ai-service/pyproject.toml`
- [ ] 1.2 Create `ai-service/models/provider.py` with Pydantic models: `ProviderMeta`, `ProviderCreateRequest`, `ProviderTestResult`, `ChatMessage`, `ChatRequest`, `ChatResponse`
- [ ] 1.3 Create `ai-service/services/provider_service.py` with `test_provider()` logic using `litellm.acompletion()` for 1-token ping
- [ ] 1.4 Create `ai-service/services/chat_service.py` with `chat_completion()` routing to `litellm.acompletion()`
- [ ] 1.5 Create `ai-service/routers/providers.py` with `POST /api/v1/providers/test` endpoint
- [ ] 1.6 Create `ai-service/routers/chat.py` with `POST /api/v1/chat/completions` endpoint
- [ ] 1.7 Modify `ai-service/main.py` to include routers and add lifespan startup for litellm pre-load

**Files created/modified:**
- `ai-service/pyproject.toml` (modify)
- `ai-service/main.py` (modify)
- `ai-service/models/provider.py` (create)
- `ai-service/services/provider_service.py` (create)
- `ai-service/services/chat_service.py` (create)
- `ai-service/routers/providers.py` (create)
- `ai-service/routers/chat.py` (create)

**Acceptance criteria:**
- `python -m uvicorn main:app --host 127.0.0.1 --port 8321` starts ≤15s
- `POST /api/v1/providers/test` returns `{success: true}` with valid OpenAI key
- `POST /api/v1/chat/completions` returns `{content, model, usage}` structure
- Invalid `provider_type` returns HTTP 422
- Error responses never include API keys

---

## Phase 2: Rust Secure Config (T2)

- [ ] 2.1 Add `keyring = "3"` to `src-tauri/Cargo.toml` dependencies
- [ ] 2.2 Create `src-tauri/src/config/models.rs` with `ProviderMeta` and `AppConfig` serde structs
- [ ] 2.3 Modify `src-tauri/src/config/mod.rs` to implement `read_config()` / `write_config()` for `AppConfig`
- [ ] 2.4 Add keyring helpers: `save_api_key(id, key)` / `get_api_key(id)` / `delete_api_key(id)` using service `"scholar-ai"`
- [ ] 2.5 Ensure `api_key` field is NEVER serialized to `config.json` — metadata only

**Files created/modified:**
- `src-tauri/Cargo.toml` (modify)
- `src-tauri/src/config/models.rs` (create)
- `src-tauri/src/config/mod.rs` (modify)

**Acceptance criteria:**
- `save_provider()` writes metadata to `config.json`, key to OS keychain
- `get_api_key()` reads from keychain, never logs the key
- Linux without `libsecret-1-0` returns `Err("KeyringUnavailable: install libsecret-1-0")`
- `config.json` contains no `api_key` field

---

## Phase 3: Rust Bridge Commands + AiBridge (T3)

- [ ] 3.1 Modify `src-tauri/src/ai_bridge/mod.rs` to add `post(path, body)` helper with JSON serialization
- [ ] 3.2 Create Tauri commands in `src-tauri/src/commands/mod.rs`: `list_providers`, `add_provider`, `update_provider`, `delete_provider`, `set_active_provider`, `test_provider`
- [ ] 3.3 Create async Tauri command `send_chat_message(provider_id, messages)` in `commands/mod.rs` that calls `AiBridge::post("/api/v1/chat/completions")`
- [ ] 3.4 Modify `src-tauri/src/lib.rs` to register new commands and pass `ConfigManager` to `AiBridge`
- [ ] 3.5 Modify `src-tauri/capabilities/default.json` to add permissions for new commands
- [ ] 3.6 Ensure snake_case field names in Rust serde structs match Pydantic models

**Files created/modified:**
- `src-tauri/src/ai_bridge/mod.rs` (modify)
- `src-tauri/src/commands/mod.rs` (modify)
- `src-tauri/src/lib.rs` (modify)
- `src-tauri/capabilities/default.json` (modify)

**Acceptance criteria:**
- `add_provider` creates keychain entry + metadata JSON
- `delete_provider` removes from both keychain and JSON
- `test_provider` invokes `POST /api/v1/providers/test` on sidecar
- `send_chat_message` returns non-streaming response ≤10s
- `Content-Length` matches body (no chunked encoding)

---

## Phase 4: Frontend ConfigPage UI (T4)

- [ ] 4.1 Create `src/stores/providers.ts` Zustand store with `providers[]`, `activeId`, `testResult` state (NO api_key persisted)
- [ ] 4.2 Create `src/hooks/useAI.ts` hook wrapping `invoke("send_chat_message")`
- [ ] 4.3 Modify `src/pages/ConfigPage.tsx` to add provider list table (name/type/model/active)
- [ ] 4.4 Add "Add Provider" form to ConfigPage: name, type select, api_base (Ollama), masked api_key, models
- [ ] 4.5 Add per-row "Test Connection" button with inline success/error badge
- [ ] 4.6 Ensure api_key stays in local component state only — NOT in localStorage/sessionStorage/Zustand persist

**Files created/modified:**
- `src/stores/providers.ts` (create)
- `src/hooks/useAI.ts` (create)
- `src/pages/ConfigPage.tsx` (modify)

**Acceptance criteria:**
- Provider list renders from `list_providers` command
- "Add Provider" form submits and updates list
- "Test Connection" shows green ✓ or red ✗ without echoing key
- App restart shows providers without api_key in state
- Delete removes provider from list and keychain

---

## Phase 5: Testing (T5)

- [ ] 5.1 Python: pytest for Pydantic model validation (valid/invalid `provider_type`)
- [ ] 5.2 Python: Mock `litellm.completion` to test `provider_service` routing logic
- [ ] 5.3 Python: FastAPI TestClient integration test for `/chat/completions` endpoint
- [ ] 5.4 Rust: Unit test for config read/write round-trip with temporary `config.json`
- [ ] 5.5 Rust: Mock sidecar HTTP to test Tauri command dispatch
- [ ] 5.6 Frontend: Vitest + React Testing Library for ConfigPage form rendering
- [ ] 5.7 Frontend: Test provider list table renders correctly with mock data

**Files created/modified:**
- `ai-service/tests/` (create test files)
- `src-tauri/src/config/mod.rs` (add `#[cfg(test)]` module)
- `src/pages/__tests__/ConfigPage.test.tsx` (create)

**Acceptance criteria:**
- All Python tests pass with `pytest`
- All Rust tests pass with `cargo test`
- Frontend tests pass with `npm test`
- Coverage includes: model validation, config round-trip, form rendering

---

## Estimated Lines Changed (per unit)

| Unit | Files | Est. Lines |
|------|-------|------------|
| T1: Python sidecar | 7 | ~735 |
| T2: Rust config | 3 | ~255 |
| T3: Rust commands | 4 | ~400 |
| T4: Frontend UI | 3 | ~580 |
| T5: Testing | 7 | ~400 |
| **Total** | **24** | **~2370** |

---

## Implementation Order & Dependencies

```
T1 (Python sidecar) ──▶ T3 (Rust commands) ──▶ T4 (Frontend UI)
        │                                               ▲
        │                                               │
        └──────────▶ T2 (Rust config) ──────────────────┘
                                │
                                ▼
                            T5 (Testing) ── runs in parallel with T3/T4
```

**Recommended PR sequence (stacked to main):**
1. **PR #1** — T1: Python sidecar LiteLLM integration
2. **PR #2** — T2: Rust secure config (keyring + models)
3. **PR #3** — T3: Rust bridge commands + AiBridge
4. **PR #4** — T4 + T5: Frontend UI + Testing

Each PR is independently reviewable and deployable.

---

**Status**: ready for implementation (sdd-apply)
**Next Step**: Begin T1 implementation (Python sidecar) via `sdd-apply`
