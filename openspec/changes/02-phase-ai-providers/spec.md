# Delta Spec: Phase 2 — AI Providers

4 NEW capabilities on top of Phase 1. Initial providers: **OpenAI** (key) and **Ollama** (no key). API keys MUST live in OS keychain — never in plain JSON, logs, or error responses.

---

## Capability 1 — `python-sidecar-ai`

**`pyproject.toml`**: `litellm>=1.0.0,<2.0.0`. **Pydantic `ProviderConfig`**: `name, provider_type ∈ {openai,ollama}, api_key?, api_base?, models[], enabled, priority`. **Bind**: `127.0.0.1:8321` only.

### Req: FastAPI endpoints

| Method | Path | Body / Response |
|---|---|---|
| `POST` | `/api/v1/chat/completions` | `{messages, model?, temperature?, max_tokens?}` → `{content, model, usage}` via `litellm.acompletion()` |
| `POST` | `/api/v1/providers/test` | `{provider_type, api_key, model}` → `{success, response?, error?}` (1-token call) |

#### Scenario: litellm importable

- GIVEN sidecar started via `python -m uvicorn main:app`
- WHEN process initializes
- THEN `import litellm` succeeds and FastAPI boots ≤15s

#### Scenario: invalid provider_type

- GIVEN `provider_type:"watson"`
- WHEN parsed
- THEN HTTP 422 `ValidationError`

#### Scenario: OpenAI round-trip

- GIVEN valid OpenAI key
- WHEN posting `{messages:[{role:user,content:"hi"}], model:"gpt-4o-mini"}`
- THEN HTTP 200 `{content, model:"gpt-4o-mini", usage}`

#### Scenario: provider error masks key

- GIVEN invalid key
- WHEN chat requested
- THEN HTTP 502 `{error:"provider_error", detail}` — NO key in body

#### Scenario: test valid

- GIVEN real key + `gpt-4o-mini`
- WHEN `/providers/test` called
- THEN `{success:true}` within 5s

#### Scenario: test invalid

- GIVEN rejected key
- WHEN `/providers/test` called
- THEN `{success:false, error:"AuthenticationError"}` HTTP 200

---

## Capability 2 — `secure-api-key-storage`

**Split**: API keys → OS keychain via `keyring = "3"` (service `"scholar-ai"`, key = provider UUID). Metadata → `config.json` (app_data_dir) — NO `api_key` field.

#### Scenario: save provider

- GIVEN `{name:"OpenAI", type:"openai", api_key:"sk-..."}`
- WHEN `save_provider` invoked
- THEN key in keychain, metadata only in JSON

#### Scenario: retrieve key

- GIVEN saved provider
- WHEN `get_provider_api_key(id)` called
- THEN key read from keychain, NEVER logged/written/sent

#### Scenario: Linux no libsecret

- GIVEN Linux host without `libsecret-1-0`
- WHEN `save_provider` runs
- THEN `Err("KeyringUnavailable: install libsecret-1-0")` — no plain-text fallback

---

## Capability 3 — `ai-provider-management`

**Tauri CRUD commands** (all `async`, `State<ConfigManager>` + `State<AiBridge>`): `list_providers`, `add_provider`, `update_provider`, `delete_provider`, `set_active_provider`, `test_provider`.

**`ConfigPage.tsx` UI**: (1) list table — name/type/model count/active toggle; (2) "Add Provider" form — name, type select, api_base (Ollama), masked api_key, models; (3) per-row "Test Connection".

#### Scenario: add + list

- GIVEN form submitted
- WHEN `add_provider` runs
- THEN provider in next `list_providers` AND keychain entry created

#### Scenario: delete

- GIVEN provider `p-123`
- WHEN `delete_provider("p-123")` runs
- THEN removed from `config.json` AND keychain

#### Scenario: test success UI

- GIVEN valid key in form
- WHEN "Test Connection" clicked
- THEN green `✓ Connected` badge

#### Scenario: test failure UI

- GIVEN invalid key
- WHEN "Test Connection" clicked
- THEN red `✗ Authentication failed` — api_key NOT cleared/echoed

#### Scenario: restart preserves no key

- GIVEN saved provider
- WHEN app restarts
- THEN provider in list, api_key field empty (keychain holds secret)

**Constraint**: key stays in local component state — NOT in `localStorage`/`sessionStorage`/Zustand persist.

---

## Capability 4 — `chat-completion`

**Path**: `src/services/ai.ts` `aiCompletion(req)` → Tauri `ai_completion` → `POST` to sidecar via `reqwest`.

**Protocol**: Rust `serde` structs mirror Pydantic models; field names `snake_case` in JSON.

#### Scenario: happy-path chat

- GIVEN active OpenAI provider
- WHEN user sends message
- THEN reply in chat panel ≤10s, `usage` in dev console only

#### Scenario: snake_case preserved

- GIVEN `max_tokens:100` from frontend
- WHEN Rust serializes
- THEN body has `max_tokens:100`, Pydantic accepts

#### Scenario: non-streaming contract

- GIVEN any chat request
- WHEN response arrives
- THEN `Content-Length` matches body, no `Transfer-Encoding: chunked`

**Constraint**: streaming is Phase 3 — MUST NOT stream in Phase 2.

---

## Dependencies & Constraints

- **Phase 1** complete (sidecar, `/health`, `ConfigManager`).
- **OS keychain** available (Linux needs `libsecret-1-0`).
- **LiteLLM library mode** only — no proxy.
- **Frontend**: no new deps; Zustand metadata-only.
- **Logging**: keys and `Authorization` headers MUST be redacted.
