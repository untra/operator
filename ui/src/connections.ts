export const MCP_CHECK_TIMEOUT_MS = 10_000;
export const MCP_DIAGNOSTIC_CLIENT = "operator-connections-check";
export const LOCAL_OPERATOR_COMMAND = "operator";
export const ACP_EDITORS = ["zed", "jetbrains", "emacs", "kiro"] as const;
export const MCP_RECIPES = ["json", "cli", "generic", "oauth"] as const;

export type Health = "green" | "yellow" | "red" | "gray";

export function apiHealth({ error, reachable }: { error: boolean; reachable: boolean }): Health {
  if (error) {
    return "red";
  }
  return reachable ? "green" : "gray";
}

export function mcpHealth({
  mounted,
  checkError,
  verified,
}: {
  mounted: boolean;
  checkError: boolean;
  verified: boolean;
}): Health {
  if (!mounted) {
    return "gray";
  }
  if (checkError) {
    return "red";
  }
  return verified ? "green" : "yellow";
}

export function connectionUrl(base: string, path: string): string {
  return new URL(path.replace(/^\//, ""), `${base.replace(/\/$/, "")}/`).href;
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

export function mcpRecipe(
  client: string,
  url: string,
  tokenUrl = new URL("/api/v1/auth/token", url).href,
): string {
  const helper = `jq -cn '{grant_type: "operator:access-key", access_key: env.OPERATOR_ACCESS_KEY}' | curl --fail --silent --show-error --max-time 8 -H 'Content-Type: application/json' --data-binary @- ${shellQuote(tokenUrl)} | jq -ce 'if (.access_token | type) == "string" then {Authorization: ("Bearer " + .access_token)} else error("Token exchange failed") end'`;
  if (client === "oauth") {
    return JSON.stringify({ mcpServers: { operator: { type: "sse", url } } }, null, 2);
  }
  const server = { type: "sse", url, headersHelper: helper };
  if (client === "cli") {
    return `claude mcp add-json --scope user operator ${shellQuote(JSON.stringify(server))}`;
  }
  if (client === "generic") {
    return `Transport: SSE\nEndpoint: ${url}\nToken endpoint: ${tokenUrl}\nPOST JSON: {"grant_type":"operator:access-key","access_key":"<your-access-key>"}\nAuthorization: Bearer <returned-access_token>\nExchange again before expires_in seconds elapse.`;
  }
  return JSON.stringify({ mcpServers: { operator: server } }, null, 2);
}

export function acpRecipe(editor: string, command: string, profile: string): string {
  const args = profile ? ["--profile", profile, "acp"] : ["acp"];
  if (editor === "emacs") {
    return `(add-to-list 'agent-shell-acp-agents\n  '(:name "operator" :command ${JSON.stringify(command)} :args (${args.map((arg) => JSON.stringify(arg)).join(" ")})))`;
  }
  if (editor === "kiro") {
    return `[[agents]]\nname = "operator"\ncommand = ${JSON.stringify(command)}\nargs = ${JSON.stringify(args)}\n`;
  }
  return JSON.stringify(
    editor === "jetbrains"
      ? { name: "operator", displayName: "Operator (Kanban Orchestrator)", command, args }
      : { agent_servers: { operator: { command, args, env: {} } } },
    null,
    2,
  );
}

export function downloadRecipe(content: string, filename: string): void {
  const url = URL.createObjectURL(new Blob([content], { type: "text/plain" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}
