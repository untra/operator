import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { getSessionVersion, subscribeSession } from "../api/adapter";
import { useHost } from "../host";
import { useApiMutation, useApiQuery, STATUS_POLL_MS } from "../api";
import {
  checkConnectionMutation,
  checkMcpMutation,
  connectionsQuery,
  currentSessionQuery,
  mcpClientsQuery,
  sectionsQuery,
} from "../api/definitions";
import { CONCEPTS } from "../concepts";
import {
  ACP_EDITORS,
  LOCAL_OPERATOR_COMMAND,
  MCP_RECIPES,
  acpRecipe,
  connectionUrl,
  downloadRecipe,
  mcpRecipe,
} from "../connections";
import { PageHeader } from "../components/PageHeader";
import { IdentityPanel } from "../components/IdentityPanel";
import { AccessKeysPanel } from "../components/AccessKeysPanel";
import styles from "./ConnectionsPage.module.css";

const CLIENT_PAGE_SIZE = 10;
const ACP_EXTENSIONS: Record<string, string> = {
  zed: "json",
  jetbrains: "json",
  emacs: "el",
  kiro: "toml",
};

export function ConnectionsPage() {
  const host = useHost();
  const sessionVersion = useSyncExternalStore(subscribeSession, getSessionVersion);
  return <Connections key={`${host.baseUrl()}:${host.profileId ?? "default"}:${sessionVersion}`} />;
}

function Connections() {
  const host = useHost();
  const session = useApiQuery(currentSessionQuery());
  const admin = session.data?.scopes.includes("admin") ?? false;
  const canExecute = session.data?.scopes.includes("execute") ?? false;
  const connections = useApiQuery(connectionsQuery(), { pollIntervalMs: STATUS_POLL_MS });
  const clients = useApiQuery(mcpClientsQuery(), {
    enabled: admin,
    pollIntervalMs: STATUS_POLL_MS,
  });
  const apiCheck = useApiMutation(checkConnectionMutation);
  const mcpCheck = useApiMutation(checkMcpMutation);
  const activeCheck = useRef<AbortController | null>(null);
  const activeApiCheck = useRef<AbortController | null>(null);
  const sections = useApiQuery(sectionsQuery());
  const section = sections.data?.find((item) => item.id === "connections");
  const [client, setClient] = useState<string>(MCP_RECIPES[0]);
  const [editor, setEditor] = useState<string>(ACP_EDITORS[0]);
  const [command, setCommand] = useState(LOCAL_OPERATOR_COMMAND);
  const [localProfile, setLocalProfile] = useState("");
  const [accessKey, setAccessKey] = useState("");
  const [feedback, setFeedback] = useState("");
  const [clientPage, setClientPage] = useState(0);
  const resetCheck = mcpCheck.reset;
  useEffect(
    () => () => {
      activeCheck.current?.abort();
      activeApiCheck.current?.abort();
      resetCheck();
    },
    [resetCheck],
  );
  const data = connections.data;
  const base = host.baseUrl();
  const publicBase = data?.public_url ?? base;
  const endpoint = data ? connectionUrl(publicBase, data.mcp_path) : "";
  const recipe = mcpRecipe(
    client,
    endpoint || base,
    connectionUrl(publicBase, "/api/v1/auth/token"),
  );
  const localRecipe = acpRecipe(editor, command, localProfile);
  const clientRows = clients.data ?? [];
  const currentClientPage = Math.min(
    clientPage,
    Math.max(0, Math.ceil(clientRows.length / CLIENT_PAGE_SIZE) - 1),
  );
  const visibleClients = clientRows.slice(
    currentClientPage * CLIENT_PAGE_SIZE,
    (currentClientPage + 1) * CLIENT_PAGE_SIZE,
  );
  const wrapper = section?.children.find((row) => row.id === "control-wrapper");

  async function copy(value: string) {
    try {
      await navigator.clipboard.writeText(value);
      setFeedback("Copied.");
    } catch {
      setFeedback("Clipboard unavailable. Select and copy the text.");
    }
  }

  function checkMcp() {
    activeCheck.current?.abort();
    const controller = new AbortController();
    activeCheck.current = controller;
    mcpCheck.mutate({ accessKey, signal: controller.signal });
    setAccessKey("");
  }

  return (
    <div className={styles.page}>
      <PageHeader
        title={CONCEPTS.connections.label}
        summary={CONCEPTS.connections.summary}
        icon={CONCEPTS.connections.icon}
        docsUrl={CONCEPTS.connections.docsUrl}
      />
      <output>{feedback}</output>
      {connections.isLoading && <p>Loading Operator connections...</p>}
      {connections.error && (
        <p role="alert">
          Connection status is stale: {connections.error.message}. Check the network or sign in
          again.
        </p>
      )}
      {session.error && <p role="alert">Unable to read permissions: {session.error.message}</p>}
      <div className={styles.grid}>
        <section className={styles.card}>
          <h2>Operator</h2>
          <p>Version {data?.version ?? "unavailable"}</p>
          <p>
            {connections.error
              ? "Unreachable or unauthorized"
              : data
                ? "API reachable"
                : "Checking API"}
          </p>
          <code>{data ? connectionUrl(base, data.api_path) : base}</code>
          {data && (
            <p className={styles.hint}>
              Profile: {data.profile_name} · Observed {new Date(data.observed_at).toLocaleString()}
            </p>
          )}
          {wrapper && <p className={styles.hint}>Control Wrapper: {wrapper.description}</p>}
          <div className={styles.actions}>
            <button
              onClick={() => {
                activeApiCheck.current?.abort();
                const controller = new AbortController();
                activeApiCheck.current = controller;
                apiCheck.mutate({ signal: controller.signal });
              }}
              disabled={apiCheck.isPending}
            >
              {apiCheck.isPending ? "Checking..." : "Check connection"}
            </button>
            <button
              disabled={!data}
              onClick={() => {
                if (data) {
                  void copy(connectionUrl(base, data.api_path));
                }
              }}
            >
              Copy API base
            </button>
            <a href={connectionUrl(base, "/swagger-ui/")} target="_blank" rel="noreferrer">
              Swagger
            </a>
            <a
              href={connectionUrl(base, "/api-docs/openapi.json")}
              target="_blank"
              rel="noreferrer"
            >
              OpenAPI
            </a>
          </div>
          {apiCheck.error && <p role="alert">{apiCheck.error.message}</p>}
          {apiCheck.data && (
            <output>
              API verified in {apiCheck.data.latencyMs} ms at{" "}
              {new Date(apiCheck.data.checkedAt).toLocaleString()}
            </output>
          )}
          <p className={styles.hint}>
            Swagger and OpenAPI require a signed-in browser. If a route is blocked, check the
            ingress configuration.
          </p>
        </section>

        <section className={styles.card}>
          <h2>MCP</h2>
          <p>
            {data
              ? data.mcp_http_mounted
                ? "SSE endpoint available"
                : "HTTP transport disabled"
              : "Transport status unavailable"}
          </p>
          <code>{endpoint}</code>
          {data && (
            <p>
              {data.mcp_connected_clients} connected clients · Profile: {data.profile_name}
            </p>
          )}
          <p className={styles.hint}>
            Local stdio: {data?.mcp_stdio_advertised ? "advertised" : "not advertised"}. Local
            processes cannot be observed here.
          </p>
          {data && !data.mcp_http_mounted && (
            <p>
              Enable mcp.http_enabled in Operator configuration and restart the API to accept remote
              clients.
            </p>
          )}
          <details>
            <summary>Configure MCP client</summary>
            <label>
              Client setup
              <select value={client} onChange={(event) => setClient(event.target.value)}>
                {MCP_RECIPES.map((value) => (
                  <option key={value} value={value}>
                    {value === "json"
                      ? "Claude Code JSON"
                      : value === "cli"
                        ? "Claude Code CLI"
                        : "Generic SSE client"}
                  </option>
                ))}
              </select>
            </label>
            <p className={styles.hint}>
              Claude Code recipes require curl and jq. Set OPERATOR_ACCESS_KEY in your client
              environment. Use user configuration (~/.claude.json), or the user-scope CLI recipe;
              project helpers may filter credential variables. The helper exchanges your key for a
              fresh bearer token on connection.
            </p>
            <pre>{recipe}</pre>
            <div className={styles.actions}>
              <button
                disabled={!data?.mcp_http_mounted}
                onClick={() => {
                  void copy(endpoint);
                }}
              >
                Copy MCP endpoint
              </button>
              <button
                disabled={!data?.mcp_http_mounted}
                onClick={() => {
                  void copy(recipe);
                }}
              >
                Copy MCP setup
              </button>
              <button
                disabled={!data?.mcp_http_mounted}
                onClick={() =>
                  downloadRecipe(
                    recipe,
                    client === "json" ? "operator-mcp.json" : "operator-mcp.txt",
                  )
                }
              >
                Download MCP setup
              </button>
            </div>
          </details>
          <a
            href="#/connections#operator-api-keys"
            onClick={(event) => {
              event.preventDefault();
              document.getElementById("operator-api-keys")?.scrollIntoView();
            }}
          >
            Manage API keys
          </a>
          <label>
            Access key for test (optional)
            <input
              type="password"
              autoComplete="off"
              value={accessKey}
              onChange={(event) => setAccessKey(event.target.value)}
            />
          </label>
          <p className={styles.hint}>
            Leave blank to test this browser session. Keys need mcp:connect; select individual MCP
            tool capabilities and their projects to permit tool calls. The test uses this browser's
            Operator address, initializes MCP, and lists tools without calling them. A separately
            configured public address must be verified from your client.
          </p>
          <button
            onClick={checkMcp}
            disabled={!data?.mcp_http_mounted || mcpCheck.isPending || (!canExecute && !accessKey)}
          >
            {mcpCheck.isPending ? "Testing handshake..." : "Test MCP connection"}
          </button>
          {mcpCheck.error && (
            <p role="alert">
              {mcpCheck.error.name === "TimeoutError"
                ? "MCP handshake timed out after 10 seconds. Check proxy streaming and connectivity."
                : mcpCheck.error.message}
            </p>
          )}
          {mcpCheck.data ? (
            <output>
              Last handshake verified using {mcpCheck.data.authentication.toLowerCase()} in{" "}
              {mcpCheck.data.latencyMs} ms at {new Date(mcpCheck.data.checkedAt).toLocaleString()}.
              This is a past observation, not a continuous check.
            </output>
          ) : (
            <p className={styles.hint}>Handshake not verified.</p>
          )}
          {admin && (
            <>
              <h3>Observed clients</h3>
              {clients.isLoading && <p>Loading clients...</p>}
              {clients.error && (
                <p role="alert">Client observations are stale: {clients.error.message}</p>
              )}
              {clients.data?.length === 0 && <p>No remote clients are connected.</p>}
              <ul>
                {visibleClients.map((item) => (
                  <li key={item.id}>
                    {item.client_name ?? "Unnamed client"} {item.client_version} ·{" "}
                    {item.initialized ? "Connected" : "Initializing"} · Last activity{" "}
                    {new Date(item.last_activity_at).toLocaleString()}
                  </li>
                ))}
              </ul>
              {clientRows.length > CLIENT_PAGE_SIZE && (
                <div className={styles.actions}>
                  <button
                    disabled={currentClientPage === 0}
                    onClick={() => setClientPage(currentClientPage - 1)}
                  >
                    Previous clients
                  </button>
                  <button
                    disabled={(currentClientPage + 1) * CLIENT_PAGE_SIZE >= clientRows.length}
                    onClick={() => setClientPage(currentClientPage + 1)}
                  >
                    Next clients
                  </button>
                </div>
              )}
            </>
          )}
        </section>

        <section className={styles.card}>
          <h2>ACP</h2>
          <p>Local status unavailable</p>
          <p className={styles.hint}>
            Your editor launches a local Operator process over stdio. Install Operator locally and
            configure a local workspace/profile first. This does not connect ACP to this hosted
            Operator.
          </p>
          <details>
            <summary>Configure local ACP editor</summary>
            <label>
              Editor
              <select value={editor} onChange={(event) => setEditor(event.target.value)}>
                {ACP_EDITORS.map((value) => (
                  <option key={value} value={value}>
                    {value}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Local Operator executable
              <input value={command} onChange={(event) => setCommand(event.target.value)} />
            </label>
            <label>
              Local profile (optional)
              <input
                value={localProfile}
                onChange={(event) => setLocalProfile(event.target.value)}
              />
            </label>
            <pre>{localRecipe}</pre>
            <div className={styles.actions}>
              <button
                disabled={!command.trim()}
                onClick={() => {
                  void copy(localRecipe);
                }}
              >
                Copy ACP setup
              </button>
              <button
                disabled={!command.trim()}
                onClick={() =>
                  downloadRecipe(localRecipe, `operator-acp.${ACP_EXTENSIONS[editor]}`)
                }
              >
                Download ACP setup
              </button>
            </div>
          </details>
        </section>
        <section className={styles.card}>
          <AccessKeysPanel />
          <IdentityPanel />
        </section>
      </div>
    </div>
  );
}
