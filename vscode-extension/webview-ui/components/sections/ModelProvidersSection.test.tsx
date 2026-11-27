import "../../test-dom";
import { afterEach, describe, expect, mock, test } from "bun:test";
import { act, cleanup, fireEvent, render } from "@testing-library/react";
import type { ExtensionToWebviewMessage, WebviewToExtensionMessage } from "../../types/messages";

const DETECTED_TOOLS = ["claude"];
const posted: WebviewToExtensionMessage[] = [];
const listeners = new Set<(msg: ExtensionToWebviewMessage) => void>();
function incoming(msg: ExtensionToWebviewMessage) {
  for (const listener of listeners) {
    listener(msg);
  }
}

mock.module("../../vscodeApi", () => ({
  postMessage: (message: WebviewToExtensionMessage) => {
    posted.push(message);
  },
  onMessage: (handler: (message: ExtensionToWebviewMessage) => void) => {
    listeners.add(handler);
    return () => {
      listeners.delete(handler);
    };
  },
}));

const { ModelProvidersSection } = await import("./ModelProvidersSection");

const KIND = {
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
};

afterEach(() => {
  cleanup();
  posted.length = 0;
});

function loadProvider(): void {
  incoming({
    type: "modelProvidersLoaded",
    kinds: [KIND],
    servers: [
      {
        name: "team-openai",
        kind: "openai",
        base_url: null,
        api_key_env: null,
        extra_env: {},
        display_name: null,
        user_declared: true,
      },
    ],
    delegators: [],
  });
  incoming({
    type: "providerProbed",
    slug: "team-openai",
    result: {
      status: "connected",
      server: "team-openai",
      reachable: true,
      models: [{ id: "gpt-test", display_name: "GPT Test" }],
      error: null,
    },
  });
}

function selectByLabel(
  getByLabelText: (label: string) => HTMLElement,
  label: string,
): HTMLSelectElement {
  const node = getByLabelText(label);
  if (!(node instanceof HTMLSelectElement)) {
    throw new Error(`expected <select> for ${label}`);
  }
  return node;
}

describe("CreateDelegatorForm selects", () => {
  test("Provider and Model start on an empty option, and Create is a no-op until both are chosen", () => {
    const view = render(<ModelProvidersSection detectedTools={DETECTED_TOOLS} apiReachable />);
    act(() => {
      loadProvider();
    });

    const provider = selectByLabel(view.getByLabelText, "Provider");
    expect(provider.value).toBe("");
    expect([...provider.options].map((option) => option.value)).toEqual(["", "team-openai"]);

    fireEvent.click(view.getByRole("button", { name: "Create delegator" }));
    expect(posted.filter((message) => message.type === "createDelegator")).toEqual([]);
    expect(view.getByRole("button", { name: "Create delegator" }).hasAttribute("disabled")).toBe(
      true,
    );

    fireEvent.change(provider, { target: { value: "team-openai" } });
    const model = selectByLabel(view.getByLabelText, "Model");
    expect(model.value).toBe("");
    expect([...model.options].map((option) => option.value)).toEqual(["", "gpt-test"]);

    fireEvent.change(model, { target: { value: "gpt-test" } });
    fireEvent.click(view.getByRole("button", { name: "Create delegator" }));
    expect(posted.filter((message) => message.type === "createDelegator")).toEqual([
      {
        type: "createDelegator",
        request: {
          name: "claude-gpt-test",
          llm_tool: "claude",
          model: "gpt-test",
          display_name: null,
          model_properties: {},
          model_server: "team-openai",
          launch_config: null,
          remote_agent: null,
          governance: null,
        },
      },
    ]);
  });
});
