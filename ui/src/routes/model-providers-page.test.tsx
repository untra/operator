import { afterEach, describe, expect, jest, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ApiProvider } from "../api";
import { resetSessionState } from "../api/adapter";
import { setCsrfToken } from "../api-client";
import { HostContext, type Host } from "../host";
import { mockFetch, restoreFetch } from "../test-fetch";
import * as webcomponentMocks from "../test-webcomponents";

mock.module("@operator/webcomponents", () => webcomponentMocks);

const { ModelProvidersPage } = await import("./ModelProvidersPage");

const TEST_HOST: Host = {
  baseUrl: () => "http://operator.test",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};

function json(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  restoreFetch();
  jest.restoreAllMocks();
});

describe("ModelProvidersPage", () => {
  test("creates and updates delegators", async () => {
    const requests: Array<{ path: string; method: string }> = [];
    const delegator = {
      name: "claude-gpt",
      llm_tool: "claude",
      model: "gpt-test",
      display_name: null,
      model_properties: {},
      model_server: "openai",
      launch_config: null,
      remote_agent: null,
      git: null,
    };
    mockFetch((input, init) => {
      const path = new URL(String(input)).pathname;
      const method = init?.method ?? "GET";
      requests.push({ path, method });
      if (path.endsWith("/model-servers/kinds")) {
        return Promise.resolve(
          json([
            {
              slug: "openai",
              display_name: "OpenAI",
              description: "OpenAI models",
              setup_url: "https://example.test",
              icon: "sparkle",
              is_builtin: true,
              category: "first-party",
              category_label: "First-party",
              brand_icon: null,
              default_base_url: "https://api.example.test",
              default_api_key_env: "OPENAI_API_KEY",
              connectable: true,
            },
          ]),
        );
      }
      if (path.endsWith("/llm-tools")) {
        return Promise.resolve(
          json({
            tools: [
              {
                name: "claude",
                version: "1",
                min_version: null,
                version_ok: true,
                model_aliases: [],
                capabilities: {},
                health_ok: true,
              },
            ],
            total: 1,
          }),
        );
      }
      if (path.endsWith("/model-servers/kinds/openai/models")) {
        return Promise.resolve(
          json({
            server: "openai",
            reachable: true,
            models: [{ id: "gpt-test", display_name: "GPT Test" }],
            error: null,
          }),
        );
      }
      if (path.endsWith("/delegators") && method === "GET") {
        return Promise.resolve(json({ delegators: [delegator], total: 1 }));
      }
      if (path.endsWith("/delegators") && method === "POST") {
        return Promise.resolve(json(delegator));
      }
      if (path.endsWith("/delegators/claude-gpt") && method === "PUT") {
        return Promise.resolve(json(delegator));
      }
      return Promise.resolve(json({ message: `Unexpected request: ${path}` }));
    });
    setCsrfToken("csrf");
    render(
      <ApiProvider>
        <HostContext.Provider value={TEST_HOST}>
          <ModelProvidersPage />
        </HostContext.Provider>
      </ApiProvider>,
    );
    await screen.findByRole("heading", { name: "Model Providers" });
    fireEvent.change(screen.getByLabelText("Provider"), { target: { value: "openai" } });
    await waitFor(() => expect(screen.getByLabelText("Model")).toBeTruthy());
    fireEvent.change(screen.getByLabelText("Model"), { target: { value: "gpt-test" } });
    fireEvent.click(screen.getByRole("button", { name: "Create delegator" }));
    await screen.findByText('Created delegator "claude-gpt-test".');
    fireEvent.click(screen.getByRole("button", { name: "Save Git settings" }));
    await waitFor(() => {
      expect(requests).toContainEqual({ path: "/api/v1/delegators", method: "POST" });
      expect(requests).toContainEqual({
        path: "/api/v1/delegators/claude-gpt",
        method: "PUT",
      });
    });
  });
});
