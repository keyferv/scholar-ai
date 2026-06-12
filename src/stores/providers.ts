import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";

export interface ProviderMeta {
  id: string;
  name: string;
  provider_type: string;
  model?: string;
  base_url?: string;
}

export interface ProviderTestResult {
  success: boolean;
  error?: string;
}

interface UpdateProviderParams {
  id: string;
  name?: string;
  providerType?: string;
  model?: string;
  baseUrl?: string;
  apiKey?: string;
  extraHeaders?: Record<string, string>;
}

interface ProvidersState {
  providers: ProviderMeta[];
  activeId: string | null;
  testResult: ProviderTestResult | null;
  fetchProviders: () => Promise<void>;
  addProvider: (
    id: string,
    name: string,
    providerType: string,
    model?: string,
    baseUrl?: string,
    apiKey?: string,
    extraHeaders?: Record<string, string>
  ) => Promise<void>;
  updateProvider: (params: UpdateProviderParams) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
  setActiveProvider: (id: string) => Promise<void>;
  testProvider: (
    providerType: string,
    model: string,
    apiKey: string,
    baseUrl?: string
  ) => Promise<void>;
  clearTestResult: () => void;
}

export const useProvidersStore = create<ProvidersState>()(
  devtools((set, get) => ({
    providers: [],
    activeId: null,
    testResult: null,

    fetchProviders: async () => {
      try {
        const result = await invoke("list_providers");
        const providers = Array.isArray(result) ? result : [];
        set({ providers });
      } catch (err) {
        console.error("Failed to fetch providers:", err);
        set({ providers: [] });
      }
    },

    addProvider: async (id, name, providerType, model, baseUrl, apiKey, extraHeaders) => {
      try {
        await invoke("add_provider", {
          id,
          name,
          provider_type: providerType,
          model: model ?? null,
          base_url: baseUrl ?? null,
          api_key: apiKey ?? null,
          extra_headers: extraHeaders ?? null,
        });
        await get().fetchProviders();
      } catch (err) {
        console.error("Failed to add provider:", err);
        throw err;
      }
    },

    updateProvider: async ({ id, name, providerType, model, baseUrl, apiKey, extraHeaders }: UpdateProviderParams) => {
      try {
        const updates: Record<string, unknown> = { id };
        if (name !== undefined) updates.name = name;
        if (providerType !== undefined) updates.provider_type = providerType;
        if (model !== undefined) updates.model = model ?? null;
        if (baseUrl !== undefined) updates.base_url = baseUrl ?? null;
        if (apiKey !== undefined) updates.api_key = apiKey ?? null;
        if (extraHeaders !== undefined) updates.extra_headers = extraHeaders ?? null;

        await invoke("update_provider", updates);
        await get().fetchProviders();
      } catch (err) {
        console.error("Failed to update provider:", err);
        throw err;
      }
    },

    deleteProvider: async (id: string) => {
      try {
        await invoke("delete_provider", { id });
        set((state) => ({
          providers: state.providers.filter((p) => p.id !== id),
          activeId: state.activeId === id ? null : state.activeId,
        }));
      } catch (err) {
        console.error("Failed to delete provider:", err);
        throw err;
      }
    },

    setActiveProvider: async (id: string) => {
      try {
        await invoke("set_active_provider", { id });
        set({ activeId: id });
      } catch (err) {
        console.error("Failed to set active provider:", err);
        throw err;
      }
    },

    testProvider: async (providerType, model, apiKey, baseUrl) => {
      try {
        const result = await invoke("test_provider", {
          provider_type: providerType,
          model,
          api_key: apiKey,
          base_url: baseUrl ?? undefined,
        });
        const parsed = typeof result === "string" ? JSON.parse(result) : result;
        set({ testResult: parsed as ProviderTestResult });
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        set({
          testResult: {
            success: false,
            error: msg.includes("detail")
              ? msg.replace(/.*detail["\s:]*/, "").trim()
              : "Test failed — check configuration",
          },
        });
      }
    },

    clearTestResult: () => set({ testResult: null }),
  }))
);