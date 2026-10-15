import { expect, test } from "bun:test";
import { testMcpConnection } from "./mcp-check";

test("refuses to send credentials to an unexpected SSE message URL", async () => {
  const sent: string[] = [];
  const send = (url: string) => {
    sent.push(url);
    return Promise.resolve(
      new Response("event: endpoint\ndata: https://attacker.test/message\n\n", {
        headers: { "Content-Type": "text/event-stream" },
      }),
    );
  };
  await expect(
    testMcpConnection("https://operator.test/api/v1/mcp/sse", send, new AbortController().signal),
  ).rejects.toThrow("unexpected message endpoint");
  expect(sent).toEqual(["https://operator.test/api/v1/mcp/sse"]);
});

test("performs initialization and discovery without invoking tools", async () => {
  const methods: string[] = [];
  let stream: ReadableStreamDefaultController<Uint8Array> | undefined;
  const encoder = new TextEncoder();
  const send = (_url: string, init?: RequestInit) => {
    if (!init?.body) {
      return Promise.resolve(
        new Response(
          new ReadableStream<Uint8Array>({
            start(controller) {
              stream = controller;
              controller.enqueue(
                encoder.encode("event: endpoint\ndata: /api/v1/mcp/message?sessionId=test\n\n"),
              );
            },
          }),
          { headers: { "Content-Type": "text/event-stream" } },
        ),
      );
    }
    const request = JSON.parse(typeof init.body === "string" ? init.body : "null");
    methods.push(request.method);
    if (request.id) {
      stream?.enqueue(
        encoder.encode(
          `event: message\ndata: ${JSON.stringify({ jsonrpc: "2.0", id: request.id, result: request.method === "initialize" ? { protocolVersion: "2024-11-05", capabilities: { tools: {} } } : { tools: [] } })}\n\n`,
        ),
      );
    }
    return Promise.resolve(new Response(null, { status: 202 }));
  };
  await testMcpConnection(
    "https://operator.test/api/v1/mcp/sse",
    send,
    new AbortController().signal,
  );
  expect(methods).toEqual(["initialize", "notifications/initialized", "tools/list"]);
});
