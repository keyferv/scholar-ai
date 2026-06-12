import { useEffect, useState } from "react";
import {
  useProvidersStore,
  type ProviderMeta,
} from "../stores/providers";

const PROVIDER_TYPES = [
  { value: "openai", label: "OpenAI" },
  { value: "anthropic", label: "Anthropic" },
  { value: "ollama", label: "Ollama" },
  { value: "openrouter", label: "OpenRouter" },
  { value: "custom", label: "Custom" },
];

export default function ConfigPage() {
  const {
    providers,
    activeId,
    testResult,
    fetchProviders,
    addProvider,
    deleteProvider,
    setActiveProvider,
    testProvider,
    clearTestResult,
  } = useProvidersStore();

  // ── Form state (never persisted beyond this component) ────────────────────
  const [name, setName] = useState("");
  const [providerType, setProviderType] = useState("openai");
  const [apiBase, setApiBase] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [formError, setFormError] = useState("");

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  // ── Helpers ────────────────────────────────────────────────────────────────
  const isOllama = providerType === "ollama";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError("");
    if (!name.trim() || !model.trim()) {
      setFormError("Name and model are required.");
      return;
    }
    if (!isOllama && !apiKey.trim()) {
      setFormError("API key is required for non-Ollama providers.");
      return;
    }

    setSubmitting(true);
    try {
      const id = crypto.randomUUID();
      await addProvider(
        id,
        name.trim(),
        providerType,
        model.trim(),
        apiBase.trim() || undefined,
        apiKey.trim() || undefined
      );
      // Clear form, but keep the type/endpoint for convenience
      setName("");
      setModel("");
      setApiKey("");
      setApiBase("");
    } catch {
      setFormError("Failed to add provider. See console for details.");
    } finally {
      setSubmitting(false);
    }
  };

const handleTest = async (p: ProviderMeta, e: React.MouseEvent) => {
    e.stopPropagation();
    clearTestResult();

    // Retrieve the stored API key via the provider's key_id
    // We pass it through the Tauri command which reads from the keyring server-side.
    // For the test we send the provider_type + model; the Rust sidecar reads the key.
    await testProvider(
      p.provider_type,
      p.model ?? "",
      "", // key is read server-side from keyring; blank here
      p.base_url ?? undefined
    );
  };

  const handleSetActive = async (id: string) => {
    await setActiveProvider(id);
  };

  const handleDelete = async (id: string, name: string) => {
    if (!window.confirm(`Delete provider "${name}"? This also removes the API key from the keychain.`)) {
      return;
    }
    await deleteProvider(id);
  };

  // ── Render ─────────────────────────────────────────────────────────────────
  return (
    <div className="max-w-5xl mx-auto">
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-1">
        AI Providers
      </h1>
      <p className="text-gray-600 dark:text-gray-400 mb-6">
        Configure and test your AI model providers. API keys are stored securely
        in the OS keychain and are never shown or persisted in the frontend.
      </p>

      {/* ── Add Provider Form ────────────────────────────────────────────── */}
      <section className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-4">
          Add Provider
        </h2>
        <form onSubmit={handleSubmit} className="grid grid-cols-1 md:grid-cols-2 gap-4">
<div className="md:col-span-2">
             <label htmlFor="provider-name" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
               Name
             </label>
             <input
               id="provider-name"
               type="text"
               value={name}
               onChange={(e) => setName(e.target.value)}
               placeholder="e.g. My OpenAI"
               className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
               required
             />
           </div>

           <div>
             <label htmlFor="provider-type" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
               Provider Type
             </label>
             <select
               id="provider-type"
               value={providerType}
               onChange={(e) => setProviderType(e.target.value)}
               className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
             >
              {PROVIDER_TYPES.map((t) => (
                <option key={t.value} value={t.value}>
                  {t.label}
                </option>
              ))}
            </select>
          </div>

          <div>
<label htmlFor="provider-model" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
               Model
             </label>
             <input
               id="provider-model"
               type="text"
               value={model}
              onChange={(e) => setModel(e.target.value)}
              placeholder={isOllama ? "llama3" : "gpt-4o"}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>

{!isOllama && (
             <div>
               <label htmlFor="provider-api-key" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                 API Key
               </label>
               <input
                 id="provider-api-key"
                 type="password"
                 value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="sk-••••••••"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                required
              />
            </div>
          )}

{isOllama && (
             <div>
               <label htmlFor="provider-api-base" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                 API Base URL
               </label>
               <input
                 id="provider-api-base"
                 type="url"
                 value={apiBase}
                onChange={(e) => setApiBase(e.target.value)}
                placeholder="http://localhost:11434"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          )}

          {isOllama && (
            <div>
              {/* Spacer for Ollama 2-column layout */}
            </div>
          )}

          {formError && (
            <p className="text-red-600 dark:text-red-400 text-sm md:col-span-2">
              {formError}
            </p>
          )}

          <div className="md:col-span-2">
            <button
              type="submit"
              disabled={submitting}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-medium rounded-md transition-colors"
            >
              {submitting ? "Adding…" : "Add Provider"}
            </button>
          </div>
        </form>
      </section>

      {/* ── Provider List Table ─────────────────────────────────────────── */}
      <section className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
        <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-100 p-6 pb-4">
          Configured Providers
        </h2>
        {providers.length === 0 ? (
          <p className="px-6 pb-6 text-gray-500 dark:text-gray-400 text-sm">
            No providers configured yet. Add one above.
          </p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-200 dark:border-gray-700">
                <th className="text-left px-4 py-3 font-medium text-gray-600 dark:text-gray-300">
                  Name
                </th>
                <th className="text-left px-4 py-3 font-medium text-gray-600 dark:text-gray-300">
                  Type
                </th>
                <th className="text-left px-4 py-3 font-medium text-gray-600 dark:text-gray-300">
                  Model
                </th>
                <th className="text-left px-4 py-3 font-medium text-gray-600 dark:text-gray-300">
                  Status
                </th>
                <th className="text-right px-4 py-3 font-medium text-gray-600 dark:text-gray-300">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              {providers.map((p) => {
                const isActive = activeId === p.id;
                return (
                  <tr
                    key={p.id}
                    className="border-b border-gray-100 dark:border-gray-700 last:border-0 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                  >
                    <td className="px-4 py-3 font-medium text-gray-900 dark:text-gray-100">
                      {p.name}
                    </td>
                    <td className="px-4 py-3 text-gray-600 dark:text-gray-400 capitalize">
                      {p.provider_type}
                    </td>
                    <td className="px-4 py-3 text-gray-600 dark:text-gray-400 font-mono text-xs">
                      {p.model ?? "—"}
                    </td>
                    <td className="px-4 py-3">
                      {/* Test result badge */}
                      {testResult &&
                        testResult.success &&
                        testResult.error === undefined ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                          ✓ Connected
                        </span>
                      ) : testResult && !testResult.success ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400">
                          ✗ {testResult.error || "Failed"}
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400">
                          Untested
                        </span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <div className="flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleTest(p, event as any)}
                          className="px-2.5 py-1 text-xs font-medium text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-md transition-colors"
                        >
                          Test
                        </button>
                        <button
                          onClick={() => handleSetActive(p.id)}
                          disabled={isActive}
                          className={`px-2.5 py-1 text-xs font-medium rounded-md transition-colors ${
                            isActive
                              ? "text-green-600 dark:text-green-400 cursor-default"
                              : "text-gray-600 hover:text-green-700 dark:text-gray-400 dark:hover:text-green-300 hover:bg-green-50 dark:hover:bg-green-900/10"
                          }`}
                        >
                          {isActive ? "Active" : "Set Active"}
                        </button>
                        <button
                          onClick={() => handleDelete(p.id, p.name)}
                          className="px-2.5 py-1 text-xs font-medium text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md transition-colors"
                        >
                          Delete
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}