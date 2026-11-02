import { describe, expect, test } from "bun:test";

import type { ModelServerModelsResponse } from "../generated/ModelServerModelsResponse";
import { probeStatusLabel } from "./ModelProviderChoices";

function probe(partial: Partial<ModelServerModelsResponse>): ModelServerModelsResponse {
  return { server: "x", reachable: false, status: "error", models: [], error: null, ...partial };
}

describe("probeStatusLabel", () => {
  test("pending probe is checking", () => {
    expect(probeStatusLabel({})).toEqual({ health: "gray", text: "checking…" });
  });

  test("failed request to operator is a probe failure", () => {
    expect(probeStatusLabel({ fetchFailed: true })).toEqual({
      health: "red",
      text: "probe failed",
    });
  });

  test("connected counts models", () => {
    const models = [{ id: "a", display_name: null }];
    expect(
      probeStatusLabel({ probe: probe({ reachable: true, status: "connected", models }) }),
    ).toEqual({ health: "green", text: "1 models" });
  });

  test("missing key names the env var", () => {
    expect(
      probeStatusLabel({ probe: probe({ status: "missing_key" }), keyEnv: "ANTHROPIC_API_KEY" }),
    ).toEqual({ health: "gray", text: "set ANTHROPIC_API_KEY" });
    expect(probeStatusLabel({ probe: probe({ status: "missing_key" }) }).text).toBe(
      "API key not set",
    );
  });

  test("unauthorized never echoes the provider body", () => {
    const label = probeStatusLabel({
      probe: probe({ status: "unauthorized", error: "server returned 401: {...}" }),
    });
    expect(label).toEqual({ health: "red", text: "API key rejected" });
  });

  test("unreachable distinguishes local servers", () => {
    expect(probeStatusLabel({ probe: probe({ status: "unreachable" }), local: true })).toEqual({
      health: "yellow",
      text: "not running",
    });
    expect(probeStatusLabel({ probe: probe({ status: "unreachable" }) }).text).toBe("unreachable");
  });

  test("other errors are a probe failure", () => {
    expect(probeStatusLabel({ probe: probe({ status: "error" }) })).toEqual({
      health: "red",
      text: "probe failed",
    });
  });
});
