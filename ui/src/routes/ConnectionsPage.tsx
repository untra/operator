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
import { HealthDot } from "@operator/webcomponents";
import { CONCEPTS, CONNECTION_CARDS, type ConnectionCard } from "../concepts";
import {
  ACP_EDITORS,
  LOCAL_OPERATOR_COMMAND,
  MCP_RECIPES,
  acpRecipe,
  apiHealth,
  connectionUrl,
  downloadRecipe,
  mcpHealth,
  mcpRecipe,
  type Health,
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

  const apiDot = apiHealth({ error: Boolean(connections.error), reachable: Boolean(data) });
  const mcpDot = mcpHealth({
    mounted: Boolean(data?.mcp_http_mounted),
    checkError: Boolean(mcpCheck.error),
    verified: Boolean(mcpCheck.data),
  });
  const meta = data
    ? [
        `Version ${data.version}`,
        `Profile ${data.profile_name}`,
        wrapper && `Wrapper ${wrapper.description}`,
        `Observed ${new Date(data.observed_at).toLocaleString()}`,
      ]
        .filter(Boolean)
        .join(" · ")
    : "";

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
        <p role="alert">Connection status is stale: {connections.error.message}</p>
      )}
      {session.error && <p role="alert">Unable to read permissions: {session.error.message}</p>}
      <div className={styles.grid}>
        <section id={CONNECTION_CARDS.operator.id} className={styles.card}>
          <CardHeader card={CONNECTION_CARDS.operator} health={apiDot} />
          {meta && <p className={styles.hint}>{meta}</p>}
          <code>{data ? connectionUrl(base, data.api_path) : base}</code>
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
              Verified · {apiCheck.data.latencyMs} ms ·{" "}
              {new Date(apiCheck.data.checkedAt).toLocaleString()}
            </output>
          )}
        </section>

        <section id={CONNECTION_CARDS.mcp.id} className={styles.card}>
          <CardHeader card={CONNECTION_CARDS.mcp} health={mcpDot} />
          {data && (
            <p className={styles.hint}>
              {data.mcp_http_mounted
                ? `SSE · ${data.mcp_connected_clients} connected clients`
                : "HTTP transport off"}
            </p>
          )}
          <code>{endpoint}</code>
          <details>
            <summary>Configure MCP client</summary>
            <label>
              Client setup
              <select value={client} onChange={(event) => setClient(event.target.value)}>
                {MCP_RECIPES.filter(
                  (value) => value !== "oauth" || data?.oauth_resource_metadata,
                ).map((value) => (
                  <option key={value} value={value}>
                    {value === "json"
                      ? "Claude Code JSON"
                      : value === "cli"
                        ? "Claude Code CLI"
                        : value === "oauth"
                          ? "OAuth (OIDC) client"
                          : "Generic SSE client"}
                  </option>
                ))}
              </select>
            </label>
            {client === "oauth" && data?.oauth_resource_metadata && (
              <p className={styles.hint}>
                The client discovers the identity provider from{" "}
                <code>{data.oauth_resource_metadata}</code> and signs in there. Only approved users
                are accepted.
              </p>
            )}
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
                    client === "json" || client === "oauth"
                      ? "operator-mcp.json"
                      : "operator-mcp.txt",
                  )
                }
              >
                Download MCP setup
              </button>
            </div>
          </details>
          <label>
            Access key for test (optional)
            <input
              type="password"
              autoComplete="off"
              value={accessKey}
              onChange={(event) => setAccessKey(event.target.value)}
            />
          </label>
          <div className={styles.actions}>
            <button
              onClick={checkMcp}
              disabled={
                !data?.mcp_http_mounted || mcpCheck.isPending || (!canExecute && !accessKey)
              }
            >
              {mcpCheck.isPending ? "Testing handshake..." : "Test MCP connection"}
            </button>
            <a
              href={`#/connections#${CONNECTION_CARDS.accessKeys.id}`}
              onClick={(event) => {
                event.preventDefault();
                document.getElementById(CONNECTION_CARDS.accessKeys.id)?.scrollIntoView();
              }}
            >
              Manage API keys
            </a>
          </div>
          {mcpCheck.error && (
            <p role="alert">
              {mcpCheck.error.name === "TimeoutError"
                ? "MCP handshake timed out after 10 seconds."
                : mcpCheck.error.message}
            </p>
          )}
          {mcpCheck.data && (
            <output>
              Verified via {mcpCheck.data.authentication.toLowerCase()} · {mcpCheck.data.latencyMs}{" "}
              ms · {new Date(mcpCheck.data.checkedAt).toLocaleString()}
            </output>
          )}
          {admin && (
            <details>
              <summary>Observed clients</summary>
              {clients.isLoading && <p>Loading clients...</p>}
              {clients.error && (
                <p role="alert">Client observations are stale: {clients.error.message}</p>
              )}
              {clients.data?.length === 0 && <p>No remote clients are connected.</p>}
              <ul>
                {visibleClients.map((item) => (
                  <li key={item.id}>
                    {item.client_name ?? "Unnamed client"} {item.client_version} ·{" "}
                    {item.initialized ? "Connected" : "Initializing"} ·{" "}
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
            </details>
          )}
        </section>

        <section id={CONNECTION_CARDS.acp.id} className={styles.card}>
          <CardHeader card={CONNECTION_CARDS.acp} health="gray" />
          <p className={styles.hint}>Local editors only</p>
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

        <section id={CONNECTION_CARDS.accessKeys.id} className={styles.card}>
          <CardHeader card={CONNECTION_CARDS.accessKeys} health="gray" />
          <AccessKeysPanel />
          <IdentityPanel />
        </section>
      </div>
    </div>
  );
}

function CardHeader({ card, health }: { card: ConnectionCard; health: Health }) {
  return (
    <header className={styles.cardHeader}>
      <HealthDot health={health} />
      <h2>{card.label}</h2>
      <a className={styles.docs} href={card.docsUrl} target="_blank" rel="noreferrer">
        Docs
      </a>
    </header>
  );
}
