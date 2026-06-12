import { describe, it, expect, vi, afterEach, type Mock } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import "@testing-library/jest-dom";
import ConfigPage from "../ConfigPage";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockedInvoke = invoke as unknown as Mock;
afterEach(() => vi.restoreAllMocks());

describe("ConfigPage", () => {
  it("renders heading and form", () => {
    mockedInvoke.mockResolvedValue([]);
    render(<ConfigPage />);
    expect(screen.getByText("AI Providers")).toBeInTheDocument();
    expect(screen.getByLabelText(/Name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Provider Type/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Model/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/API Key/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Add Provider/i })).toBeInTheDocument();
  });

  it("renders provider list from invoke", async () => {
    mockedInvoke.mockResolvedValue([
      { id: "p1", name: "OpenAI", provider_type: "openai", model: "gpt-4o", base_url: null },
    ]);
    render(<ConfigPage />);
    expect(await screen.findByText("OpenAI")).toBeInTheDocument();
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Actions")).toBeInTheDocument();
  });

  it("shows 'Untested' badge", async () => {
    mockedInvoke.mockResolvedValue([
      { id: "p1", name: "Test", provider_type: "openai", model: "gpt-4o", base_url: null },
    ]);
    render(<ConfigPage />);
    expect(await screen.findByText("Untested")).toBeInTheDocument();
  });

  it("adds a provider", async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    render(<ConfigPage />);

    await act(async () => {
      fireEvent.change(screen.getByLabelText(/Name/i), { target: { value: "New" } });
      fireEvent.change(screen.getByLabelText(/Model/i), { target: { value: "gpt-4o" } });
      fireEvent.change(screen.getByLabelText(/API Key/i), { target: { value: "sk-test" } });
      fireEvent.click(screen.getByRole("button", { name: /Add Provider/i }));
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("add_provider", expect.objectContaining({ name: "New" }));
    });
  });

  it("shows validation error on empty name/model", async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    render(<ConfigPage />);

    await act(async () => {
      fireEvent.change(screen.getByLabelText(/Name/i), { target: { value: "" } });
      fireEvent.change(screen.getByLabelText(/Model/i), { target: { value: "" } });
      fireEvent.click(screen.getByRole("button", { name: /Add Provider/i }));
    });

    expect(screen.getByText(/Name and model are required/i)).toBeInTheDocument();
  });

  it("hides API Key for Ollama, shows API Base", async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    render(<ConfigPage />);

    await act(async () => {
      fireEvent.change(screen.getByLabelText(/Provider Type/i), { target: { value: "ollama" } });
    });

    expect(screen.queryByLabelText(/API Key/i)).not.toBeInTheDocument();
    expect(screen.getByLabelText(/API Base URL/i)).toBeInTheDocument();
  });

  it("tests provider success", async () => {
    mockedInvoke
      .mockResolvedValueOnce([{ id: "p1", name: "Test", provider_type: "openai", model: "gpt-4o", base_url: null }])
      .mockResolvedValueOnce(JSON.stringify({ success: true }));
    render(<ConfigPage />);
    await waitFor(() => screen.getByText("Untested"));

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /Test/i }));
    });
    expect(await screen.findByText(/Connected/i)).toBeInTheDocument();
  });

  it("tests provider failure", async () => {
    mockedInvoke
      .mockResolvedValueOnce([{ id: "p1", name: "Test", provider_type: "openai", model: "gpt-4o", base_url: null }])
      .mockResolvedValueOnce(JSON.stringify({ success: false, error: "Bad key" }));
    render(<ConfigPage />);
    await waitFor(() => screen.getByText("Untested"));

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /Test/i }));
    });
    expect(await screen.findByText(/✗/i)).toBeInTheDocument();
  });

  it("sets active provider", async () => {
    mockedInvoke
      .mockResolvedValueOnce([{ id: "p1", name: "Test", provider_type: "openai", model: "gpt-4o", base_url: null }])
      .mockResolvedValueOnce("Active provider set to 'p1'");
    render(<ConfigPage />);
    await waitFor(() => screen.getByText("Untested"));

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /Set Active/i }));
    });
    expect(await screen.findByRole("button", { name: /Active/i })).toBeInTheDocument();
  });

  it("deletes with confirmation", async () => {
    mockedInvoke
      .mockResolvedValueOnce([
        { id: "p1", name: "A", provider_type: "openai", model: "gpt-4o", base_url: null },
        { id: "p2", name: "B", provider_type: "ollama", model: "llama3", base_url: null },
      ])
      .mockResolvedValueOnce("Provider 'p2' deleted successfully");
    vi.stubGlobal("confirm", () => true);
    render(<ConfigPage />);
    await waitFor(() => screen.getByText("Untested"));

    const deleteBtns = screen.getAllByRole("button", { name: /Delete/i });
    await act(async () => { fireEvent.click(deleteBtns[1]); });
    expect(invoke).toHaveBeenCalledWith("delete_provider", { id: "p2" });
  });

  it("skips delete on cancel", async () => {
    mockedInvoke.mockResolvedValueOnce([
      { id: "p1", name: "A", provider_type: "openai", model: "gpt-4o", base_url: null },
    ]);
    vi.stubGlobal("confirm", () => false);
    render(<ConfigPage />);
    await waitFor(() => screen.getByText("Untested"));

    const deleteBtns = screen.getAllByRole("button", { name: /Delete/i });
    await act(async () => { fireEvent.click(deleteBtns[0]); });
    expect(mockedInvoke.mock.calls.filter((c) => c[0] === "delete_provider").length).toBe(0);
  });

  it("shows empty state when no providers", async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    render(<ConfigPage />);
    expect(await screen.findByText(/No providers configured yet/i)).toBeInTheDocument();
  });
});