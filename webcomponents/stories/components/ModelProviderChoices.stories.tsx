import type { Meta, StoryObj } from "@storybook/react-vite";
import {
  ModelProviderChoices,
  type ModelProviderChoice,
} from "../../src/components/ModelProviderChoices";
import type { ModelServerModelsResponse } from "../../src/generated/ModelServerModelsResponse";
import type { ProbeStatus } from "../../src/generated/ProbeStatus";

function probe(server: string, status: ProbeStatus, count = 0, error?: string) {
  return {
    server,
    status,
    reachable: status === "connected",
    models: Array.from({ length: count }, (_, i) => ({ id: `${server}-${i}`, display_name: null })),
    error: error ?? null,
  } satisfies ModelServerModelsResponse;
}

const PROVIDERS = [
  { slug: "anthropic-api", label: "Anthropic", keyEnv: "ANTHROPIC_API_KEY" },
  { slug: "openai-api", label: "OpenAI", keyEnv: "OPENAI_API_KEY" },
  { slug: "google-api", label: "Google", keyEnv: "GEMINI_API_KEY" },
  { slug: "xai-api", label: "xAI", keyEnv: "XAI_API_KEY" },
  { slug: "ollama", label: "Ollama", local: true },
  { slug: "openrouter", label: "OpenRouter", keyEnv: "OPENROUTER_API_KEY" },
] satisfies ModelProviderChoice[];

function withStatus(
  pick: (provider: ModelProviderChoice) => ModelProviderChoice,
): ModelProviderChoice[] {
  return PROVIDERS.map(pick);
}

const ALL_CONNECTED = withStatus((p) => ({ ...p, probe: probe(p.slug, "connected", 12) }));

const meta = {
  title: "Components/ModelProviderChoices",
  component: ModelProviderChoices,
  args: { providers: ALL_CONNECTED, selected: [], onToggle: () => {} },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-form">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ModelProviderChoices>;

export default meta;
type Story = StoryObj<typeof meta>;

export const AllConnected: Story = {};

/** A fresh host: no keys exported, no local Ollama, only OpenRouter's public listing answers. */
export const NothingConfigured: Story = {
  args: {
    providers: withStatus((p) => ({
      ...p,
      probe:
        p.slug === "openrouter"
          ? probe(p.slug, "connected", 452)
          : p.slug === "ollama"
            ? probe(p.slug, "unreachable", 0, "request failed: connection refused")
            : probe(p.slug, "missing_key", 0, `${p.keyEnv} is not set`),
    })),
  },
};

/** Keys are set but the providers reject them. */
export const KeyRejected: Story = {
  args: {
    providers: withStatus((p) => ({
      ...p,
      probe: p.keyEnv
        ? probe(p.slug, "unauthorized", 0, "server returned 401: invalid x-api-key")
        : probe(p.slug, "connected", 3),
    })),
  },
};

export const LocalServerDown: Story = {
  args: {
    providers: withStatus((p) => ({
      ...p,
      probe: p.local
        ? probe(p.slug, "unreachable", 0, "request failed: connection refused")
        : probe(p.slug, "connected", 12),
    })),
  },
};

export const Checking: Story = { args: { providers: PROVIDERS } };

/** Operator's own API failed, so no probe result exists. */
export const ProbeFetchFailed: Story = {
  args: { providers: withStatus((p) => ({ ...p, fetchFailed: true })) },
};

export const ProbeError: Story = {
  args: {
    providers: withStatus((p) => ({
      ...p,
      probe: probe(p.slug, "error", 0, "could not parse model list: expected value"),
    })),
  },
};

export const MixedWithSelection: Story = {
  args: {
    providers: withStatus((p) => ({
      ...p,
      probe:
        p.slug === "anthropic-api"
          ? probe(p.slug, "connected", 9)
          : p.slug === "openai-api"
            ? probe(p.slug, "unauthorized")
            : p.slug === "ollama"
              ? probe(p.slug, "unreachable")
              : probe(p.slug, "missing_key"),
    })),
    selected: ["anthropic-api", "openai-api"],
  },
};

export const DarkTheme: Story = {
  args: NothingConfigured.args,
  globals: { theme: "dark" },
};
