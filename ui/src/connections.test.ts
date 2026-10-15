import { describe, expect, test } from "bun:test";
import { connectionUrl, mcpRecipe, acpRecipe, apiHealth, mcpHealth } from "./connections";

describe("connection setup", () => {
  test("resolves paths against the active host including its prefix", () => {
    expect(connectionUrl("https://operator.test/prefix", "/swagger-ui/")).toBe(
      "https://operator.test/prefix/swagger-ui/",
    );
    expect(connectionUrl("https://operator.test", "/api/v1/profiles/team/mcp/sse")).toBe(
      "https://operator.test/api/v1/profiles/team/mcp/sse",
    );
  });

  test("MCP config uses SSE and a credential placeholder", () => {
    const recipe = JSON.parse(mcpRecipe("json", "https://operator.test/api/v1/mcp/sse"));
    expect(recipe.mcpServers.operator.type).toBe("sse");
    expect(recipe.mcpServers.operator.url).toBe("https://operator.test/api/v1/mcp/sse");
    expect(recipe.mcpServers.operator.headersHelper).toContain("operator:access-key");
    expect(recipe.mcpServers.operator.headersHelper).toContain("env.OPERATOR_ACCESS_KEY");
    expect(recipe.mcpServers.operator.headers).toBeUndefined();
  });

  test("ACP uses the user's executable and local profile, including escaped arguments", () => {
    const recipe = JSON.parse(acpRecipe("zed", "/Applications/Operator CLI", "local team"));
    expect(recipe.agent_servers.operator.command).toBe("/Applications/Operator CLI");
    expect(recipe.agent_servers.operator.args).toEqual(["--profile", "local team", "acp"]);
    expect(acpRecipe("emacs", 'operator"test', "")).toContain('operator\\"test');
  });
});

describe("card health", () => {
  test("API is gray while loading, red on error, green once reachable", () => {
    expect(apiHealth({ error: false, reachable: false })).toBe("gray");
    expect(apiHealth({ error: true, reachable: true })).toBe("red");
    expect(apiHealth({ error: false, reachable: true })).toBe("green");
  });

  test("MCP is gray when unmounted, yellow until verified, red on a failed check", () => {
    expect(mcpHealth({ mounted: false, checkError: true, verified: false })).toBe("gray");
    expect(mcpHealth({ mounted: true, checkError: false, verified: false })).toBe("yellow");
    expect(mcpHealth({ mounted: true, checkError: false, verified: true })).toBe("green");
    expect(mcpHealth({ mounted: true, checkError: true, verified: false })).toBe("red");
  });
});
