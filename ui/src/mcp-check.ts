import { MCP_DIAGNOSTIC_CLIENT } from "./connections";

const MCP_PROTOCOL_VERSION = "2024-11-05";
const MAX_EVENT_BYTES = 1_048_576;
const INITIALIZE_ID = 1;
const TOOLS_ID = 2;

function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export async function testMcpConnection(
  endpoint: string,
  send: (url: string, init?: RequestInit) => Promise<Response>,
  signal: AbortSignal,
): Promise<void> {
  const response = await send(endpoint, { headers: { Accept: "text/event-stream" }, signal });
  if (!response.headers.get("Content-Type")?.includes("text/event-stream") || !response.body) {
    throw new Error("Endpoint did not return an SSE stream. Check the ingress route.");
  }
  const reader = response.body.getReader();
  const cancel = () => {
    void reader.cancel().catch(() => undefined);
  };
  signal.addEventListener("abort", cancel, { once: true });
  const decoder = new TextDecoder();
  let buffer = "";

  async function event(): Promise<{ type: string; data: string }> {
    for (;;) {
      signal.throwIfAborted();
      buffer = buffer.replaceAll("\r\n", "\n");
      const boundary = buffer.indexOf("\n\n");
      if (boundary >= 0) {
        const block = buffer.slice(0, boundary);
        buffer = buffer.slice(boundary + 2);
        const lines = block.split("\n");
        const data = lines
          .filter((line) => line.startsWith("data:"))
          .map((line) => line.slice(5).trimStart())
          .join("\n");
        if (data) {
          return {
            type:
              lines
                .find((line) => line.startsWith("event:"))
                ?.slice(6)
                .trim() ?? "message",
            data,
          };
        }
        continue;
      }
      const chunk = await reader.read();
      signal.throwIfAborted();
      if (chunk.done) {
        throw new Error("MCP stream closed before the handshake completed.");
      }
      buffer += decoder.decode(chunk.value, { stream: true });
      if (buffer.length > MAX_EVENT_BYTES) {
        throw new Error("MCP event exceeds the diagnostic size limit.");
      }
    }
  }

  async function reply(id: number): Promise<Record<string, unknown>> {
    for (;;) {
      const item = await event();
      if (item.type !== "message") {
        continue;
      }
      const value: unknown = JSON.parse(item.data);
      if (!object(value) || value.id !== id) {
        continue;
      }
      if (value.error) {
        throw new Error("MCP protocol request failed. Check server logs.");
      }
      if (value.jsonrpc !== "2.0" || !object(value.result)) {
        throw new Error("Invalid MCP protocol response.");
      }
      return value.result;
    }
  }

  try {
    const first = await event();
    if (first.type !== "endpoint") {
      throw new Error("MCP stream did not advertise a message endpoint.");
    }
    const message = new URL(first.data, endpoint);
    const expected = new URL(endpoint);
    expected.pathname = expected.pathname.replace(/\/sse$/, "/message");
    if (
      message.origin !== expected.origin ||
      message.pathname !== expected.pathname ||
      message.username ||
      message.password ||
      message.hash
    ) {
      throw new Error(
        "MCP advertised an unexpected message endpoint. Check the public URL configuration.",
      );
    }
    const post = (method: string, params: object, id?: number) =>
      send(message.href, {
        method: "POST",
        signal,
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          method,
          params,
          ...(id === undefined ? {} : { id }),
        }),
      });
    await post(
      "initialize",
      {
        protocolVersion: MCP_PROTOCOL_VERSION,
        capabilities: {},
        clientInfo: { name: MCP_DIAGNOSTIC_CLIENT, version: "1" },
      },
      INITIALIZE_ID,
    );
    const initialized = await reply(INITIALIZE_ID);
    if (initialized.protocolVersion !== MCP_PROTOCOL_VERSION || !object(initialized.capabilities)) {
      throw new Error(
        "MCP initialization returned an unsupported protocol version or capabilities.",
      );
    }
    await post("notifications/initialized", {});
    await post("tools/list", {}, TOOLS_ID);
    const tools = await reply(TOOLS_ID);
    if (!Array.isArray(tools.tools)) {
      throw new Error("MCP tool discovery returned an invalid response.");
    }
  } finally {
    signal.removeEventListener("abort", cancel);
    await reader.cancel().catch(() => undefined);
    reader.releaseLock();
  }
}
