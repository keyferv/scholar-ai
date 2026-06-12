## Exploration: Phase 2 AI Providers

### Current State

Phase 1 is complete and functional. The architecture consists of:
- **Frontend**: React 19 + TypeScript + Tailwind CSS + Vite, with React Router and Zustand for state management. Four placeholder pages (Dashboard, Search, Reports, Config).
- **Rust Backend (Tauri v2)**: Spawns a Python sidecar, manages SQLite DB (papers, searches, projects, reports), and has a stub ConfigManager.
- **Python Sidecar**: FastAPI + uvicorn on port 8321. Currently only exposes a `/health` endpoint. No AI provider integration yet.
- **Database**: SQLite with schema for papers, searches, search_results, projects, project_papers, and reports.

### What Already Exists (Reusable)

1. **Rust `AiBridge`**: Spawns sidecar, health polling, HTTP client (`reqwest`), graceful stop. Can be extended for provider-specific HTTP calls.
2. **Rust `ConfigManager`**: Scaffold exists with `app_data_dir` path. Needs read/write logic.
3. **Rust `Database`**: SQLite connection established. Can add a `providers` table or use `ConfigManager` for provider configs.
4. **Frontend `ConfigPage`**: Placeholder page ready for provider settings UI.
5. **Tauri capabilities**: `shell:allow-spawn` and `core:default` already configured.
6. **Python FastAPI scaffold**: `main.py` and `pyproject.toml` ready. Need to add `litellm` dependency.
7. **Zustand**: Already installed in frontend for state management — perfect for storing provider settings locally.
8. **React Router**: Already set up with `/config` route.

### What Needs to Be Built

1. **Python Sidecar — AI Provider Layer**
   - Add `litellm` to `pyproject.toml` dependencies.
   - Create a provider router: `/api/v1/providers` to list, add, test providers.
   - Create a chat completion endpoint: `/api/v1/chat/completions` that proxies to `litellm.completion()`.
   - Implement provider-specific routing (OpenAI, Anthropic, local Ollama, etc.).
   - Add error handling and fallback logic in Python.

2. **Rust Bridge — AI Commands**
   - Add Tauri commands for: `get_providers`, `add_provider`, `update_provider`, `delete_provider`, `test_provider`, `send_chat_message`.
   - The Rust bridge should NOT call OpenAI directly; it should call the Python sidecar over HTTP (keeping all AI logic in one place).
   - Extend `ai_bridge/mod.rs` with generic `post()` helper for sidecar communication.

3. **Rust Configuration — Secure API Key Storage**
   - **Critical Decision**: API keys must NOT be stored in plain JSON. Use the OS credential store.
   - Add `keyring` crate to `Cargo.toml` for cross-platform secure storage.
   - Store provider metadata (name, model, endpoint, is_active) in `config.json`.
   - Store API keys in OS keychain via `keyring`, keyed by provider ID.
   - Alternatively, encrypt keys with a user password (more secure, more UX friction). Recommendation: start with `keyring`.

4. **Frontend — Provider Config Panel**
   - Build `ConfigPage` with sections for provider management.
   - Create a form for adding/editing providers: provider name, type (OpenAI, Anthropic, Ollama, etc.), base URL, model, API key input.
   - Add a "Test Connection" button that calls `test_provider` command.
   - Add a provider list/table with active/inactive toggle.
   - Use Zustand for local state (pending changes before saving).
   - Implement a primary provider selector (fallback ordering).

5. **Frontend — AI Service Integration**
   - Create a `useAI()` hook or Zustand store for sending chat requests.
   - Add a simple chat interface on `SearchPage` or `DashboardPage` to test AI providers.
   - Implement streaming support if possible (LiteLLM supports streaming).

### Technical Decisions Needed

| Decision | Options | Recommendation |
|----------|---------|----------------|
| **API Key Storage** | Plain JSON config, SQLite encrypted, OS keychain (`keyring`) | **OS keychain (`keyring`)** — best security/UX tradeoff |
| **Provider Config Format** | JSON file, SQLite table, TOML | **JSON file** (readable, easy to backup) for metadata; **keyring** for secrets |
| **LiteLLM Mode** | Library (`litellm.completion`) vs Proxy (`litellm --proxy`) | **Library** — simpler, single process, no extra port management |
| **Fallback Strategy** | Automatic retry next provider, manual fallback, no fallback | **Manual fallback** — start with one active provider; add retry later |
| **Cost Tracking** | LiteLLM built-in, custom DB table, skip for now | **Skip for now** — add in Phase 3 when usage is real |
| **Rust ↔ Python Protocol** | Shared structs (Pydantic + serde), loose JSON | **Shared Pydantic models** with JSON schema for validation |

### Dependencies & Prerequisites

**Python:**
- `litellm>=1.0.0` (core library)
- `pydantic` (already present) — define request/response models

**Rust:**
- `keyring = "3"` (credential store)
- `reqwest` already present (for sidecar HTTP)
- `serde` + `serde_json` already present

**Frontend:**
- No new packages needed (Zustand + React + Tailwind sufficient).
- Optional: `react-hook-form` for form validation, `lucide-react` for icons.

### Affected Areas

- `ai-service/main.py` — add provider routes and LiteLLM integration
- `ai-service/pyproject.toml` — add `litellm` dependency
- `src-tauri/src/ai_bridge/mod.rs` — extend HTTP helpers and add AI commands
- `src-tauri/src/config/mod.rs` — implement read/write and keyring integration
- `src-tauri/src/commands/mod.rs` — add provider management commands
- `src-tauri/Cargo.toml` — add `keyring` dependency
- `src/pages/ConfigPage.tsx` — build provider settings UI
- `src/` — add new hooks/stores for AI interaction
- `src-tauri/src/db/mod.rs` — consider adding `providers` table if DB storage preferred over JSON

### Risks

1. **API Key Security**: If keys are stored in plain text, the app is a security liability. Must use `keyring` or equivalent.
2. **LiteLLM Version Compatibility**: LiteLLM API changes frequently. Pin to a stable version and test.
3. **Sidecar Startup Time**: Adding LiteLLM import will slow down Python sidecar startup. Monitor health-check timeout.
4. **Cross-Platform Keyring**: `keyring` works on macOS/Windows/Linux, but Linux requires `libsecret` or `dbus`. Need to document or handle gracefully.
5. **Error Handling**: Provider failures (rate limits, invalid keys) must be handled gracefully and shown to the user.
6. **Streaming Complexity**: If streaming is implemented, both Rust (HTTP client) and frontend (EventSource/fetch) need streaming support. Start with non-streaming to reduce complexity.
7. **No Tests**: The codebase currently has no tests. Adding provider logic without tests increases regression risk.

### Recommendation

**Approach**: Implement a two-layer provider system:
1. **Python sidecar** owns all AI provider logic via `litellm` (single source of truth).
2. **Rust bridge** acts as a thin HTTP proxy + secure config manager (keyring + JSON).
3. **Frontend** provides a clean config UI and uses Zustand for state.

**Effort**: Medium (2-3 focused sessions).

**Phase 2 Scope**: Start with support for 1-2 providers (e.g., OpenAI and a local Ollama) to prove the architecture. Do not build cost tracking or advanced fallback yet.

### Ready for Proposal

**Yes.** The exploration is complete. The orchestrator should proceed to `sdd-propose` for this change. The proposal should define:
- Intent: Multi-provider AI support for ScholarAI
- Scope: LiteLLM integration, provider config UI, secure API key storage, Rust bridge commands
- Approach: Python sidecar owns AI logic; Rust manages secure config; Frontend provides UI
- Out-of-scope: Cost tracking, advanced fallback, streaming (for Phase 3)

### Skill Resolution

- **sdd-explore**: ✅ Complete — this artifact
- **sdd-propose**: ⏭️ Next — create `openspec/changes/02-phase-ai-providers/proposal.md`
- **sdd-spec**: ⏭️ Pending — write delta specs for provider API, config storage, and UI
- **sdd-design**: ⏭️ Pending — design Rust commands, Python endpoints, and frontend state flow
- **sdd-tasks**: ⏭️ Pending — break into implementation tasks
- **sdd-apply**: ⏭️ Pending — implement
- **sdd-verify**: ⏭️ Pending — test
- **sdd-archive**: ⏭️ Pending — after completion
