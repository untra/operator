import { describe, expect, it } from "bun:test";
import {
  findResourceInstance,
  runTerraformApply,
  runTerraformInit,
  testRequiredVariables,
} from "~test";

describe("operator", async () => {
  await runTerraformInit(import.meta.dir);

  testRequiredVariables(import.meta.dir, {
    agent_id: "foo",
  });

  it("offline and use_cached cannot both be true", () => {
    const t = async () => {
      await runTerraformApply(import.meta.dir, {
        agent_id: "foo",
        offline: "true",
        use_cached: "true",
      });
    };
    expect(t).toThrow("offline and use_cached cannot both be true");
  });

  it("rejects invalid session_wrapper values", () => {
    const t = async () => {
      await runTerraformApply(import.meta.dir, {
        agent_id: "foo",
        session_wrapper: "invalid",
      });
    };
    expect(t).toThrow("session_wrapper must be one of");
  });

  it("rejects invalid share values", () => {
    const t = async () => {
      await runTerraformApply(import.meta.dir, {
        agent_id: "foo",
        share: "invalid",
      });
    };
    expect(t).toThrow("share must be one of");
  });

  it("applies with default values", async () => {
    const state = await runTerraformApply(import.meta.dir, {
      agent_id: "foo",
    });

    const script = findResourceInstance(state, "coder_script");
    expect(script.run_on_start).toBe(true);
    expect(script.display_name).toBe("Operator");

    const app = findResourceInstance(state, "coder_app");
    expect(app.url).toBe("http://localhost:7008");
    expect(app.slug).toBe("operator");
    expect(app.display_name).toBe("Operator");
    expect(app.share).toBe("owner");
    expect(app.healthcheck[0].url).toBe(
      "http://localhost:7008/api/v1/health",
    );
  });

  it("applies with custom port", async () => {
    const state = await runTerraformApply(import.meta.dir, {
      agent_id: "foo",
      port: "9000",
    });

    const app = findResourceInstance(state, "coder_app");
    expect(app.url).toBe("http://localhost:9000");
    expect(app.healthcheck[0].url).toBe(
      "http://localhost:9000/api/v1/health",
    );
  });

  it("generates config with custom values", async () => {
    const state = await runTerraformApply(import.meta.dir, {
      agent_id: "foo",
      port: "9000",
      max_parallel_agents: "4",
      session_wrapper: "zellij",
    });

    const script = findResourceInstance(state, "coder_script").script;
    expect(script).toContain("port = 9000");
    expect(script).toContain("max_parallel = 4");
    expect(script).toContain('wrapper = "zellij"');
  });

  it("uses config_toml verbatim when provided", async () => {
    const customConfig = '[rest_api]\nenabled = true\nport = 8080';
    const state = await runTerraformApply(import.meta.dir, {
      agent_id: "foo",
      config_toml: customConfig,
    });

    const script = findResourceInstance(state, "coder_script").script;
    expect(script).toContain(customConfig);
  });
});
