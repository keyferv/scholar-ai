import { invoke } from "@tauri-apps/api/core";

/**
 * Send a chat completion request via the Tauri "send_chat_message" command.
 *
 * @param providerId  - The ID of the provider to use for the chat.
 * @param messages    - Array of message objects with `role` and `content`.
 * @returns Parsed JSON response from the sidecar with content, model, and usage.
 */
export async function sendChatMessage(
  providerId: string,
  messages: Array<{ role: string; content: string }>
): Promise<{
  content: string;
  model: string;
  usage?: Record<string, number>;
  provider_type?: string;
}> {
  const result = await invoke("send_chat_message", {
    provider_id: providerId,
    messages,
  });

  // The Tauri command returns a JSON string from the HTTP response body.
  if (typeof result === "string") {
    return JSON.parse(result);
  }
  return result as {
    content: string;
    model: string;
    usage?: Record<string, number>;
    provider_type?: string;
  };
}