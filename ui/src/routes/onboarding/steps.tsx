import { useCallback, useEffect, useMemo, useState } from "react";
import type { KanbanProviderKind } from "@operator/bindings/KanbanProviderKind";
import type { SetupStep } from "@operator/bindings/SetupStep";
import type { StepComponent, StepProps } from "./types";
import styles from "./OnboardingPage.module.css";

const TASK_FIELDS = ["priority", "points", "user_story"] as const;
const WRAPPERS = ["tmux", "vscode", "cmux", "zellij"] as const;
const KANBAN_KINDS = ["jira", "linear", "github", "openspec"] as const;
const COLLECTION_SOURCES = [
  ["simple", "Simple"],
  ["dev_kanban", "Development"],
  ["devops_kanban", "DevOps"],
  ["custom", "Hosted collections"],
] as const;

function Intro({ children }: { children: React.ReactNode }) {
  return <div className={styles.intro}>{children}</div>;
}

function Choice<Value extends string>({
  selected,
  value,
  onSelect,
  children,
}: {
  selected: boolean;
  value: Value;
  onSelect: (value: Value) => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      className={`${styles.choice} ${selected ? styles.selected : ""}`}
      onClick={() => onSelect(value)}
    >
      {children}
    </button>
  );
}

function ExportBlock({ value }: { value: string }) {
  return (
    <div className={styles.export}>
      <strong>Make this permanent in your shell profile</strong>
      <pre>{value}</pre>
      <button type="button" onClick={() => navigator.clipboard.writeText(value)}>
        Copy exports
      </button>
    </div>
  );
}

const Welcome: StepComponent = ({ status }) => (
  <Intro>
    <h2>Welcome to Operator</h2>
    <p>We’ll configure this workspace for both the terminal and browser.</p>
    <dl>
      <dt>Configuration</dt>
      <dd>{status.config_path}</dd>
      <dt>Tickets</dt>
      <dd>{status.tickets_path}</dd>
    </dl>
    {Object.entries(status.projects_by_tool).map(([tool, projects]) => (
      <p key={tool}>
        <strong>{tool}</strong>: {projects.join(", ") || "none"}
      </p>
    ))}
  </Intro>
);

function KanbanInfo({ api, addExport }: StepProps) {
  const [providers, setProviders] = useState<Awaited<ReturnType<typeof api.kanbanProviders>>>([]);
  const [provider, setProvider] = useState<KanbanProviderKind | "">("");
  const [domain, setDomain] = useState("");
  const [email, setEmail] = useState("");
  const [token, setToken] = useState("");
  const [rootPath, setRootPath] = useState("");
  const [instance, setInstance] = useState("default");
  const [projects, setProjects] = useState<{ id: string; key: string; name: string }[]>([]);
  const [projectKey, setProjectKey] = useState("");
  const [statuses, setStatuses] = useState<string[]>([]);
  const [mapping, setMapping] = useState({ todo: "", doing: "", done: "" });
  const [syncUserId, setSyncUserId] = useState("");
  const [workspaceKey, setWorkspaceKey] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void api
      .kanbanProviders()
      .then(setProviders)
      .catch((error: Error) => setMessage(error.message));
  }, [api]);

  const selectProvider = useCallback(
    (slug: string) => setProvider(KANBAN_KINDS.find((kind) => kind === slug) ?? ""),
    [setProvider],
  );

  const credentials = () => ({
    provider: provider as KanbanProviderKind,
    jira: provider === "jira" ? { domain, email, api_token: token } : null,
    linear: provider === "linear" ? { api_key: token } : null,
    github: provider === "github" ? { token } : null,
    openspec: provider === "openspec" ? { root_path: rootPath } : null,
  });

  async function connect() {
    if (!provider) {
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      const validation = await api.validateKanbanCredentials(credentials());
      if (!validation.valid) {
        throw new Error(validation.error ?? "Credentials were rejected");
      }
      setSyncUserId(
        validation.jira?.account_id ??
          validation.linear?.user_id ??
          validation.github?.user_id ??
          "",
      );
      setWorkspaceKey(validation.github?.user_login ?? "");
      if (provider === "openspec") {
        await api.writeKanbanConfig({
          provider,
          openspec: { instance, root_path: rootPath, project: null },
          jira: null,
          linear: null,
          github: null,
        });
        setMessage("OpenSpec connected.");
        return;
      }
      const listed = await api.listKanbanProjects(credentials());
      setProjects(listed.projects);
      setMessage("Credentials validated. Choose a project.");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Connection failed");
    } finally {
      setBusy(false);
    }
  }

  async function chooseProject(value: string) {
    setProjectKey(value);
    if (!provider || !value) {
      return;
    }
    try {
      const result = await api.listKanbanStatuses({ ...credentials(), project_key: value });
      setStatuses(result.statuses);
      setMapping({
        todo: result.statuses[0] ?? "",
        doing: result.statuses[1] ?? "",
        done: result.statuses.at(-1) ?? "",
      });
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Could not list statuses");
    }
  }

  async function save() {
    if (!provider || !projectKey) {
      return;
    }
    setBusy(true);
    try {
      const env =
        provider === "jira"
          ? "OPERATOR_JIRA_API_KEY"
          : provider === "linear"
            ? "OPERATOR_LINEAR_API_KEY"
            : "OPERATOR_GITHUB_TOKEN";
      const status_mapping = mapping;
      if (provider === "jira") {
        const envResult = await api.setKanbanSessionEnv({
          provider,
          jira: { domain, email, api_token: token, api_key_env: env },
          linear: null,
          github: null,
        });
        await api.writeKanbanConfig({
          provider,
          jira: {
            domain,
            email,
            api_key_env: env,
            project_key: projectKey,
            sync_user_id: syncUserId,
            status_mapping,
          },
          linear: null,
          github: null,
          openspec: null,
        });
        addExport(envResult.shell_export_block);
      } else if (provider === "linear") {
        const envResult = await api.setKanbanSessionEnv({
          provider,
          linear: { api_key: token, api_key_env: env },
          jira: null,
          github: null,
        });
        const selected = projects.find((item) => item.key === projectKey);
        await api.writeKanbanConfig({
          provider,
          linear: {
            workspace_key: selected?.id ?? projectKey,
            api_key_env: env,
            project_key: projectKey,
            sync_user_id: syncUserId,
            status_mapping,
          },
          jira: null,
          github: null,
          openspec: null,
        });
        addExport(envResult.shell_export_block);
      } else if (provider === "github") {
        const envResult = await api.setKanbanSessionEnv({
          provider,
          github: { token, api_key_env: env },
          jira: null,
          linear: null,
        });
        const selected = projects.find((item) => item.key === projectKey);
        const owner = selected?.name.split("/#", 1)[0] ?? workspaceKey;
        await api.writeKanbanConfig({
          provider,
          github: {
            owner,
            api_key_env: env,
            project_key: selected?.id ?? projectKey,
            sync_user_id: syncUserId,
            status_mapping,
          },
          jira: null,
          linear: null,
          openspec: null,
        });
        addExport(envResult.shell_export_block);
      }
      setToken("");
      setMessage("Kanban provider connected.");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Could not save provider");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Intro>
      <h2>Kanban</h2>
      <p>Connect a board now, or continue and connect one later.</p>
      <div className={styles.choices}>
        {providers.map((item) => (
          <Choice
            key={item.slug}
            selected={provider === item.slug}
            value={item.slug}
            onSelect={selectProvider}
          >
            <strong>{item.display_name}</strong>
            <span>{item.description}</span>
          </Choice>
        ))}
      </div>
      {provider && (
        <div className={styles.form}>
          {provider === "jira" && (
            <>
              <label>
                Jira domain
                <input
                  value={domain}
                  onChange={(event) => setDomain(event.target.value)}
                  placeholder="org.atlassian.net"
                />
              </label>
              <label>
                Email
                <input value={email} onChange={(event) => setEmail(event.target.value)} />
              </label>
            </>
          )}
          {provider === "openspec" ? (
            <>
              <label>
                Instance name
                <input value={instance} onChange={(event) => setInstance(event.target.value)} />
              </label>
              <label>
                OpenSpec root
                <input value={rootPath} onChange={(event) => setRootPath(event.target.value)} />
              </label>
            </>
          ) : (
            <label>
              API token
              <input
                type="password"
                value={token}
                onChange={(event) => setToken(event.target.value)}
              />
            </label>
          )}
          <button type="button" onClick={connect} disabled={busy}>
            Validate and connect
          </button>
          {projects.length > 0 && (
            <>
              <label>
                Project
                <select value={projectKey} onChange={(event) => chooseProject(event.target.value)}>
                  <option value="">Choose…</option>
                  {projects.map((item) => (
                    <option key={item.id} value={item.key}>
                      {item.name}
                    </option>
                  ))}
                </select>
              </label>
              {statuses.length > 0 && (
                <div className={styles.mapping}>
                  {(["todo", "doing", "done"] as const).map((state) => (
                    <label key={state}>
                      {state}
                      <select
                        value={mapping[state]}
                        onChange={(event) =>
                          setMapping((current) => ({ ...current, [state]: event.target.value }))
                        }
                      >
                        {statuses.map((status) => (
                          <option key={status}>{status}</option>
                        ))}
                      </select>
                    </label>
                  ))}
                </div>
              )}
              <button type="button" onClick={save} disabled={busy || !projectKey}>
                Save provider
              </button>
            </>
          )}
          {message && <p>{message}</p>}
        </div>
      )}
    </Intro>
  );
}

function ModelServer({ api, integrations, draft, setDraft }: StepProps) {
  const entries = useMemo(
    () => integrations.filter((entry) => entry.vertical === "model"),
    [integrations],
  );
  const [kinds, setKinds] = useState<Awaited<ReturnType<typeof api.listProviderKinds>>>([]);
  const [probes, setProbes] = useState<Record<string, string>>({});
  useEffect(() => {
    let active = true;
    void api.listProviderKinds().then((result) => {
      if (active) {
        setKinds(result);
      }
      return undefined;
    });
    for (const entry of entries) {
      void api
        .providerModels(entry.slug)
        .then((result) => {
          if (active) {
            setProbes((current) => ({
              ...current,
              [entry.slug]: result.reachable
                ? `${result.models.length} models`
                : (result.error ?? "unreachable"),
            }));
          }
          return undefined;
        })
        .catch(() => {
          if (active) {
            setProbes((current) => ({ ...current, [entry.slug]: "unreachable" }));
          }
        });
    }
    return () => {
      active = false;
    };
  }, [api, entries]);
  const keyExports = draft.modelServers.flatMap((slug) => {
    const env = kinds.find((kind) => kind.slug === slug)?.default_api_key_env;
    return env ? [`export ${env}="<your-token>"`] : [];
  });
  const toggleProvider = useCallback(
    (provider: string) =>
      setDraft((current) => ({
        ...current,
        modelServers: current.modelServers.includes(provider)
          ? current.modelServers.filter((slug) => slug !== provider)
          : [...current.modelServers, provider],
      })),
    [setDraft],
  );
  return (
    <Intro>
      <h2>Model providers</h2>
      <p>
        Select the providers this workspace uses. Operator stores environment-variable names, never
        API keys.
      </p>
      <div className={styles.choices}>
        {entries.map((entry) => (
          <Choice
            key={entry.slug}
            selected={draft.modelServers.includes(entry.slug)}
            value={entry.slug}
            onSelect={toggleProvider}
          >
            <strong>{entry.label}</strong>
            <span>{probes[entry.slug] ?? "checking…"}</span>
          </Choice>
        ))}
      </div>
      {keyExports.length > 0 && <ExportBlock value={keyExports.join("\n")} />}
    </Intro>
  );
}

function GitProvider({ api, addExport }: StepProps) {
  const [providers, setProviders] = useState<Awaited<ReturnType<typeof api.gitProviders>>>([]);
  const [selected, setSelected] = useState("");
  const [token, setToken] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    void api
      .gitProviders()
      .then(setProviders)
      .catch((error: Error) => setMessage(error.message));
  }, [api]);
  const provider = providers.find((item) => item.slug === selected);
  async function save() {
    if (!provider) {
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      if (provider.state !== "authenticated") {
        const validation = await api.validateGitToken({ provider: provider.slug, token });
        if (!validation.valid) {
          throw new Error(validation.error ?? "Token was rejected");
        }
      }
      const config = await api.writeGitConfig({
        provider: provider.slug,
        token_env: provider.token_env,
      });
      if (token) {
        const env = await api.setGitSessionEnv({ provider: provider.slug, token });
        addExport(env.shell_export_block);
      } else {
        addExport(config.shell_export_block);
      }
      setToken("");
      setMessage(`Connected ${provider.label}${config.username ? ` as ${config.username}` : ""}.`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Could not connect provider");
    } finally {
      setBusy(false);
    }
  }
  return (
    <Intro>
      <h2>Git provider</h2>
      <p>Choose a catalog provider. Existing CLI authentication is adopted when available.</p>
      <div className={styles.choices}>
        {providers.map((item) => (
          <Choice
            key={item.slug}
            selected={selected === item.slug}
            value={item.slug}
            onSelect={setSelected}
          >
            <strong>{item.label}</strong>
            <span>
              {item.state === "authenticated"
                ? `authenticated${item.username ? ` as ${item.username}` : ""}`
                : item.command}
            </span>
          </Choice>
        ))}
      </div>
      {provider && (
        <div className={styles.form}>
          {provider.state !== "authenticated" && (
            <label>
              Personal access token
              <input
                type="password"
                value={token}
                onChange={(event) => setToken(event.target.value)}
              />
            </label>
          )}
          <a href={provider.action_url} target="_blank" rel="noreferrer">
            Provider setup
          </a>
          <button
            type="button"
            disabled={busy || (provider.state !== "authenticated" && !token)}
            onClick={save}
          >
            Connect
          </button>
          {message && <p>{message}</p>}
        </div>
      )}
    </Intro>
  );
}

const CollectionSource: StepComponent = ({ draft, setDraft }) => {
  const selectPreset = useCallback(
    (preset: (typeof COLLECTION_SOURCES)[number][0]) =>
      setDraft((current) => ({ ...current, preset })),
    [setDraft],
  );
  return (
    <Intro>
      <h2>Issue type collection</h2>
      <div className={styles.choices}>
        {COLLECTION_SOURCES.map(([value, label]) => (
          <Choice
            key={value}
            selected={draft.preset === value}
            value={value}
            onSelect={selectPreset}
          >
            <strong>{label}</strong>
          </Choice>
        ))}
      </div>
    </Intro>
  );
};

const HostedCollections: StepComponent = ({ collections, draft, setDraft }) => {
  const toggleCollection = useCallback(
    (id: string) =>
      setDraft((current) => ({
        ...current,
        hostedCollectionIds: current.hostedCollectionIds.includes(id)
          ? current.hostedCollectionIds.filter((collectionId) => collectionId !== id)
          : [...current.hostedCollectionIds, id],
      })),
    [setDraft],
  );
  return (
    <Intro>
      <h2>Hosted collections</h2>
      <p>Select one or more. The checksum locks initialization to the version you reviewed.</p>
      <div className={styles.choices}>
        {collections.map((item) => (
          <Choice
            key={item.id}
            selected={draft.hostedCollectionIds.includes(item.id)}
            value={item.id}
            onSelect={toggleCollection}
          >
            <strong>{item.name}</strong>
            <span>{item.description}</span>
            <small>{item.types.join(", ")}</small>
          </Choice>
        ))}
      </div>
    </Intro>
  );
};

const TaskFieldConfig: StepComponent = ({ draft, setDraft }) => {
  const toggleTaskField = useCallback(
    (field: string) =>
      setDraft((current) => ({
        ...current,
        taskFields: current.taskFields.includes(field)
          ? current.taskFields.filter((item) => item !== field)
          : [...current.taskFields, field],
      })),
    [setDraft],
  );
  return (
    <Intro>
      <h2>Optional task fields</h2>
      <div className={styles.choices}>
        {TASK_FIELDS.map((field) => (
          <Choice
            key={field}
            selected={draft.taskFields.includes(field)}
            value={field}
            onSelect={toggleTaskField}
          >
            <strong>{field.replace("_", " ")}</strong>
          </Choice>
        ))}
      </div>
    </Intro>
  );
};

const SessionWrapperChoice: StepComponent = ({ draft, setDraft }) => {
  const selectWrapper = useCallback(
    (wrapper: (typeof WRAPPERS)[number]) =>
      setDraft((current) => ({
        ...current,
        wrapper,
        executionTarget:
          wrapper === "zellij" && current.executionTarget.kind === "coder"
            ? { kind: "local" }
            : current.executionTarget,
      })),
    [setDraft],
  );
  return (
    <Intro>
      <h2>Session wrapper</h2>
      <div className={styles.choices}>
        {WRAPPERS.map((wrapper) => (
          <Choice
            key={wrapper}
            selected={draft.wrapper === wrapper}
            value={wrapper}
            onSelect={selectWrapper}
          >
            <strong>{wrapper}</strong>
          </Choice>
        ))}
      </div>
    </Intro>
  );
};

const ExecutionTarget: StepComponent = ({ draft, setDraft }) => {
  const selectExecutionTarget = useCallback(
    (kind: "local" | "coder") =>
      setDraft((current) => ({
        ...current,
        useWorktrees: kind === "coder" ? false : current.useWorktrees,
        executionTarget:
          kind === "local"
            ? { kind: "local" }
            : {
                kind: "coder",
                name: "coder-agents",
                template: "",
                parameters: {},
              },
      })),
    [setDraft],
  );
  return (
    <Intro>
      <h2>Execution target</h2>
      <div className={styles.choices}>
        <Choice
          selected={draft.executionTarget.kind === "local"}
          value="local"
          onSelect={selectExecutionTarget}
        >
          <strong>Local</strong>
          <span>Run beside Operator</span>
        </Choice>
        <Choice
          selected={draft.executionTarget.kind === "coder"}
          value="coder"
          onSelect={selectExecutionTarget}
        >
          <strong>Coder</strong>
          <span>One workspace per ticket over SSH</span>
        </Choice>
      </div>
      {draft.executionTarget.kind === "coder" && (
        <div className={styles.form}>
          <label>
            Target name
            <input
              value={draft.executionTarget.name}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  executionTarget: {
                    kind: "coder",
                    name: event.target.value,
                    template:
                      current.executionTarget.kind === "coder"
                        ? current.executionTarget.template
                        : "",
                    parameters:
                      current.executionTarget.kind === "coder"
                        ? current.executionTarget.parameters
                        : {},
                  },
                }))
              }
            />
          </label>
          <label>
            Coder template
            <input
              value={draft.executionTarget.template}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  executionTarget: {
                    kind: "coder",
                    name:
                      current.executionTarget.kind === "coder"
                        ? current.executionTarget.name
                        : "coder-agents",
                    template: event.target.value,
                    parameters:
                      current.executionTarget.kind === "coder"
                        ? current.executionTarget.parameters
                        : {},
                  },
                }))
              }
            />
          </label>
          <fieldset className={styles.parameters}>
            <legend>Template parameters (optional)</legend>
            {draft.coderParameters.map((parameter, index) => (
              <div className={styles.parameterRow} key={parameter.id}>
                <input
                  aria-label={`Coder parameter ${index + 1} name`}
                  placeholder="name"
                  value={parameter.name}
                  onChange={(event) =>
                    setDraft((current) => ({
                      ...current,
                      coderParameters: current.coderParameters.map((item) =>
                        item.id === parameter.id ? { ...item, name: event.target.value } : item,
                      ),
                    }))
                  }
                />
                <input
                  aria-label={`Coder parameter ${index + 1} value`}
                  placeholder="value"
                  value={parameter.value}
                  onChange={(event) =>
                    setDraft((current) => ({
                      ...current,
                      coderParameters: current.coderParameters.map((item) =>
                        item.id === parameter.id ? { ...item, value: event.target.value } : item,
                      ),
                    }))
                  }
                />
                <button
                  type="button"
                  onClick={() =>
                    setDraft((current) => ({
                      ...current,
                      coderParameters: current.coderParameters.filter(
                        (item) => item.id !== parameter.id,
                      ),
                    }))
                  }
                >
                  Remove
                </button>
              </div>
            ))}
            <button
              type="button"
              onClick={() =>
                setDraft((current) => ({
                  ...current,
                  coderParameters: [
                    ...current.coderParameters,
                    {
                      id:
                        current.coderParameters.reduce(
                          (highest, parameter) => Math.max(highest, parameter.id),
                          0,
                        ) + 1,
                      name: "",
                      value: "",
                    },
                  ],
                }))
              }
            >
              Add parameter
            </button>
          </fieldset>
          <p>Set CODER_URL and CODER_SESSION_TOKEN in the server environment.</p>
        </div>
      )}
    </Intro>
  );
};

const WorktreePreference: StepComponent = ({ draft, setDraft }) => {
  const selectWorktreePreference = useCallback(
    (preference: "in-place" | "worktree") =>
      setDraft((current) => ({ ...current, useWorktrees: preference === "worktree" })),
    [setDraft],
  );
  return (
    <Intro>
      <h2>Git worktrees</h2>
      <p>Coder targets always isolate work remotely, so local worktrees are disabled for them.</p>
      <div className={styles.choices}>
        <Choice selected={!draft.useWorktrees} value="in-place" onSelect={selectWorktreePreference}>
          <strong>In-place branches</strong>
        </Choice>
        <Choice selected={draft.useWorktrees} value="worktree" onSelect={selectWorktreePreference}>
          <strong>Per-ticket worktrees</strong>
        </Choice>
      </div>
    </Intro>
  );
};

const AdminPassword: StepComponent = () => (
  <Intro>
    <h2>Admin account</h2>
    <p>
      Your browser session is authenticated. The password was configured before this workspace
      wizard opened.
    </p>
  </Intro>
);
const TmuxOnboarding: StepComponent = () => (
  <Intro>
    <h2>tmux</h2>
    <p>
      Operator will launch each agent in its own tmux session. Install tmux and keep it available on
      PATH.
    </p>
  </Intro>
);
const VSCodeSetup: StepComponent = () => (
  <Intro>
    <h2>VS Code</h2>
    <p>Install the Operator extension to launch and follow agent terminals from VS Code.</p>
  </Intro>
);
const CmuxSetup: StepComponent = () => (
  <Intro>
    <h2>cmux</h2>
    <p>Run Operator inside cmux so launched workspaces can be focused from the dashboard.</p>
  </Intro>
);
const ZellijSetup: StepComponent = () => (
  <Intro>
    <h2>Zellij</h2>
    <p>
      Operator will create a Zellij session for each agent. Coder targets are not compatible with
      this wrapper.
    </p>
  </Intro>
);
const AcceptanceCriteria: StepComponent = ({ draft, setDraft }) => (
  <Intro>
    <h2>Acceptance criteria</h2>
    <textarea
      className={styles.editor}
      value={draft.acceptanceCriteria}
      onChange={(event) =>
        setDraft((current) => ({ ...current, acceptanceCriteria: event.target.value }))
      }
    />
  </Intro>
);
const StartupTickets: StepComponent = () => (
  <Intro>
    <h2>Startup tickets</h2>
    <p>
      You can create onboarding and project tickets later from the dashboard. Ticket creation is
      read-only in this first web release.
    </p>
  </Intro>
);
const Confirm: StepComponent = ({ status, draft, exports }) => (
  <Intro>
    <h2>Ready to initialize</h2>
    <dl>
      <dt>Destination</dt>
      <dd>{status.config_path}</dd>
      <dt>Collection</dt>
      <dd>{draft.preset}</dd>
      <dt>Wrapper</dt>
      <dd>{draft.wrapper}</dd>
      <dt>Execution</dt>
      <dd>{draft.executionTarget.kind}</dd>
      {draft.executionTarget.kind === "coder" && (
        <>
          <dt>Coder parameters</dt>
          <dd>{draft.coderParameters.length} stored in the project configuration</dd>
        </>
      )}
      <dt>Worktrees</dt>
      <dd>{draft.useWorktrees ? "enabled" : "disabled"}</dd>
    </dl>
    {exports.map((value) => (
      <ExportBlock key={value} value={value} />
    ))}
  </Intro>
);

export const STEP_COMPONENTS = {
  welcome: Welcome,
  "kanban-info": KanbanInfo,
  "model-server": ModelServer,
  "git-provider": GitProvider,
  "collection-source": CollectionSource,
  "hosted-collections": HostedCollections,
  "task-field-config": TaskFieldConfig,
  "session-wrapper-choice": SessionWrapperChoice,
  "execution-target": ExecutionTarget,
  "worktree-preference": WorktreePreference,
  "admin-password": AdminPassword,
  "tmux-onboarding": TmuxOnboarding,
  "vscode-setup": VSCodeSetup,
  "cmux-setup": CmuxSetup,
  "zellij-setup": ZellijSetup,
  "acceptance-criteria": AcceptanceCriteria,
  "startup-tickets": StartupTickets,
  confirm: Confirm,
} satisfies Record<SetupStep, StepComponent>;

export function visibleSteps(steps: SetupStep[], draft: StepProps["draft"]): SetupStep[] {
  const wrapperStep: Record<StepProps["draft"]["wrapper"], SetupStep> = {
    tmux: "tmux-onboarding",
    vscode: "vscode-setup",
    cmux: "cmux-setup",
    zellij: "zellij-setup",
  };
  const wrapperSteps = new Set<SetupStep>(Object.values(wrapperStep));
  return steps.filter(
    (step) =>
      (step !== "hosted-collections" || draft.preset === "custom") &&
      (!wrapperSteps.has(step) || step === wrapperStep[draft.wrapper]),
  );
}
