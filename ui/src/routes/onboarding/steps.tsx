import { useCallback, useMemo, useRef, useState } from "react";
import type { KanbanProviderKind } from "@operator/bindings/KanbanProviderKind";
import type { SetupStep } from "@operator/bindings/SetupStep";
import type { StepComponent, StepProps, StepRow } from "./types";
import { Choice, ChoiceGroup, PremiumPaywall } from "@operator/webcomponents";
import { LicensePanel } from "../../components/LicensePanel";
import { useApiMutation, useApiQuery } from "../../api";
import {
  gitProvidersQuery,
  kanbanProvidersQuery,
  licenseQuery,
  listKanbanProjectsMutation,
  listKanbanStatusesMutation,
  providerKindsQuery,
  providerModelsQuery,
  setGitSessionEnvMutation,
  setKanbanSessionEnvMutation,
  validateGitTokenMutation,
  validateKanbanCredentialsMutation,
  writeGitConfigMutation,
  writeKanbanConfigMutation,
} from "../../api/definitions";
import styles from "./OnboardingPage.module.css";

const TASK_FIELDS = ["priority", "points", "user_story"] as const;
const WRAPPERS = ["tmux", "vscode", "cmux", "zellij"] as const;
const KANBAN_KINDS = ["jira", "linear", "github", "openspec"] as const;
const EMPTY_STATUS_MAPPING = { todo: "", doing: "", done: "" };
const COLLECTION_SOURCES = [
  ["simple", "Simple"],
  ["dev_kanban", "Development"],
  ["devops_kanban", "DevOps"],
  ["custom", "Hosted collections"],
] as const;

function Intro({ children }: { children: React.ReactNode }) {
  return <div className={styles.intro}>{children}</div>;
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

const Welcome: StepComponent = ({ status, creating, draft, setDraft }) => (
  <Intro>
    <h2>Welcome to Operator</h2>
    <p>We’ll configure this workspace for both the terminal and browser.</p>
    <label className={styles.form}>
      Configuration name
      <input
        required
        pattern="[a-z0-9_-]+"
        maxLength={64}
        value={draft.configurationName}
        onChange={(event) =>
          setDraft((current) => ({ ...current, configurationName: event.target.value }))
        }
      />
      <span>Use lowercase letters, numbers, hyphens, and underscores.</span>
    </label>
    {!creating && (
      <>
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
      </>
    )}
  </Intro>
);

const License: StepComponent = () => <LicensePanel />;

const ExecutionMode: StepComponent = ({ draft, setDraft }) => {
  const licenseQueryResult = useApiQuery(licenseQuery());
  const [showLicense, setShowLicense] = useState(false);
  const license = licenseQueryResult.data;

  const selectLocal = useCallback(() => {
    setDraft((current) => ({
      ...current,
      executionMode: "local",
      executionTarget: { kind: "local" },
    }));
  }, [setDraft]);

  const selectRemote = useCallback(() => {
    if (!license?.premium) {
      setShowLicense(true);
      return;
    }
    setDraft((current) => ({
      ...current,
      executionMode: "remote",
      useWorktrees: false,
      executionTarget:
        current.executionTarget.kind === "coder"
          ? current.executionTarget
          : { kind: "coder", name: "coder-agents", template: "", parameters: {} },
    }));
  }, [license?.premium, setDraft]);

  const addLicense = useCallback(() => setShowLicense(true), []);

  return (
    <Intro>
      <h2>Where will agents run?</h2>
      <p>Both choices support multiple agents. Remote targets require Premium.</p>
      <ChoiceGroup>
        <Choice value="local" selected={draft.executionMode === "local"} onSelect={selectLocal}>
          <strong>This machine</strong>
          <span>Run agents and local containers beside Operator.</span>
        </Choice>
        <Choice value="remote" selected={draft.executionMode === "remote"} onSelect={selectRemote}>
          <strong>Remote targets · Premium</strong>
          <span>Launch remotely and report work back to this Operator server.</span>
        </Choice>
      </ChoiceGroup>
      {!license?.premium && (
        <PremiumPaywall
          inline
          feature="Remote targets"
          purchaseUrl={license?.purchase_url}
          onAddLicense={addLicense}
        />
      )}
      {showLicense && <LicensePanel />}
    </Intro>
  );
};

function KanbanInfo({ addExport }: StepProps) {
  const providersQuery = useApiQuery(kanbanProvidersQuery());
  const validateCredentials = useApiMutation(validateKanbanCredentialsMutation);
  const listProjects = useApiMutation(listKanbanProjectsMutation);
  const listStatuses = useApiMutation(listKanbanStatusesMutation);
  const writeConfig = useApiMutation(writeKanbanConfigMutation);
  const setSessionEnv = useApiMutation(setKanbanSessionEnvMutation);
  const [provider, setProvider] = useState<KanbanProviderKind | "">("");
  const [domain, setDomain] = useState("");
  const [email, setEmail] = useState("");
  const [token, setToken] = useState("");
  const [rootPath, setRootPath] = useState("");
  const [instance, setInstance] = useState("default");
  const [projectKey, setProjectKey] = useState("");
  const [mapping, setMapping] = useState(EMPTY_STATUS_MAPPING);
  const statusRequest = useRef(0);
  const [syncUserId, setSyncUserId] = useState("");
  const [workspaceKey, setWorkspaceKey] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const providers = providersQuery.data ?? [];
  const projects = listProjects.data?.projects ?? [];
  const statuses = listStatuses.data?.statuses ?? [];
  const busy =
    validateCredentials.isPending ||
    listProjects.isPending ||
    listStatuses.isPending ||
    writeConfig.isPending ||
    setSessionEnv.isPending;

  const selectProvider = useCallback(
    (slug: string) => {
      setProvider(KANBAN_KINDS.find((kind) => kind === slug) ?? "");
      setProjectKey("");
      setMapping(EMPTY_STATUS_MAPPING);
      statusRequest.current += 1;
      listProjects.reset();
      listStatuses.reset();
    },
    [listProjects, listStatuses],
  );

  // The catalog carries one more provider than `KANBAN_KINDS`: the built-in board
  const builtInBoard = providers.find((item) => !KANBAN_KINDS.some((kind) => kind === item.slug));
  const connectable = providers.filter((item) => KANBAN_KINDS.some((kind) => kind === item.slug));

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
    setMessage(null);
    try {
      const validation = await validateCredentials.mutateAsync(credentials());
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
        await writeConfig.mutateAsync({
          provider,
          openspec: { instance, root_path: rootPath, project: null },
          jira: null,
          linear: null,
          github: null,
        });
        setMessage("OpenSpec connected.");
        return;
      }
      await listProjects.mutateAsync(credentials());
      setMessage("Credentials validated. Choose a project.");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Connection failed");
    }
  }

  async function chooseProject(value: string) {
    const request = ++statusRequest.current;
    setProjectKey(value);
    setMapping(EMPTY_STATUS_MAPPING);
    if (!provider || !value) {
      return;
    }
    try {
      const result = await listStatuses.mutateAsync({ ...credentials(), project_key: value });
      if (request !== statusRequest.current) {
        return;
      }
      setMapping({
        todo: result.statuses[0] ?? "",
        doing: result.statuses[1] ?? "",
        done: result.statuses.at(-1) ?? "",
      });
    } catch (error) {
      if (request !== statusRequest.current) {
        return;
      }
      setMessage(error instanceof Error ? error.message : "Could not list statuses");
    }
  }

  async function save() {
    if (!provider || !projectKey) {
      return;
    }
    try {
      const env =
        provider === "jira"
          ? "OPERATOR_JIRA_API_KEY"
          : provider === "linear"
            ? "OPERATOR_LINEAR_API_KEY"
            : "OPERATOR_GITHUB_TOKEN";
      const status_mapping = mapping;
      if (provider === "jira") {
        const envResult = await setSessionEnv.mutateAsync({
          provider,
          jira: { domain, email, api_token: token, api_key_env: env },
          linear: null,
          github: null,
        });
        await writeConfig.mutateAsync({
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
        const envResult = await setSessionEnv.mutateAsync({
          provider,
          linear: { api_key: token, api_key_env: env },
          jira: null,
          github: null,
        });
        const selected = projects.find((item) => item.key === projectKey);
        await writeConfig.mutateAsync({
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
        const envResult = await setSessionEnv.mutateAsync({
          provider,
          github: { token, api_key_env: env },
          jira: null,
          linear: null,
        });
        const selected = projects.find((item) => item.key === projectKey);
        const owner = selected?.name.split("/#", 1)[0] ?? workspaceKey;
        await writeConfig.mutateAsync({
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
    }
  }

  return (
    <Intro>
      <h2>Kanban</h2>
      <p>
        Connect an external Kanban provider to sync its issues, or continue and connect one later.
      </p>
      <ChoiceGroup>
        {builtInBoard && (
          <Choice
            key={builtInBoard.slug}
            value={builtInBoard.slug}
            selected
            locked
            wide
            onSelect={selectProvider}
          >
            <strong>{builtInBoard.display_name}</strong>
            <span>
              Built in and already active - your tickets in <code>.tickets/</code> are the board.
            </span>
          </Choice>
        )}
        {connectable.map((item) => (
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
      </ChoiceGroup>
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
                <select
                  value={projectKey}
                  disabled={listStatuses.isPending}
                  onChange={(event) => chooseProject(event.target.value)}
                >
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
          {(message ?? providersQuery.error?.message) && (
            <p>{message ?? providersQuery.error?.message}</p>
          )}
        </div>
      )}
    </Intro>
  );
}

function ProbeLabel({ slug }: { slug: string }) {
  const { data, error, isLoading } = useApiQuery(providerModelsQuery(slug));
  if (data?.reachable) {
    return `${data.models.length} models`;
  }
  if (isLoading) {
    return "checking…";
  }
  return data?.error ?? (error ? "unreachable" : "checking…");
}

function ModelServer({ integrations, draft, setDraft }: StepProps) {
  const entries = useMemo(
    () => integrations.filter((entry) => entry.vertical === "model"),
    [integrations],
  );
  const kindsQuery = useApiQuery(providerKindsQuery());
  const kinds = kindsQuery.data ?? [];
  const keyExports = draft.modelServers.flatMap((slug) => {
    const env = kinds.find((kind) => kind.slug === slug)?.default_api_key_env;
    return env ? [`export ${env}="<your-token>"`] : [];
  });
  const toggleProvider = useCallback(
    (provider: string) =>
      setDraft((current) => ({
        ...current,
        modelServers: current.modelServers.includes(provider)
          ? current.modelServers.filter((item) => item !== provider)
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
      <ChoiceGroup>
        {entries.map((entry) => (
          <Choice
            key={entry.slug}
            selected={draft.modelServers.includes(entry.slug)}
            value={entry.slug}
            onSelect={toggleProvider}
          >
            <strong>{entry.label}</strong>
            <span>
              <ProbeLabel slug={entry.slug} />
            </span>
          </Choice>
        ))}
      </ChoiceGroup>
      {keyExports.length > 0 && <ExportBlock value={keyExports.join("\n")} />}
    </Intro>
  );
}

function GitProvider({ addExport }: StepProps) {
  const providersQuery = useApiQuery(gitProvidersQuery());
  const validateToken = useApiMutation(validateGitTokenMutation);
  const writeConfig = useApiMutation(writeGitConfigMutation);
  const setSessionEnv = useApiMutation(setGitSessionEnvMutation);
  const [selected, setSelected] = useState("");
  const [token, setToken] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const providers = providersQuery.data ?? [];
  const busy = validateToken.isPending || writeConfig.isPending || setSessionEnv.isPending;
  const provider = providers.find((item) => item.slug === selected);
  async function save() {
    if (!provider) {
      return;
    }
    setMessage(null);
    try {
      if (provider.state !== "authenticated") {
        const validation = await validateToken.mutateAsync({ provider: provider.slug, token });
        if (!validation.valid) {
          throw new Error(validation.error ?? "Token was rejected");
        }
      }
      const config = await writeConfig.mutateAsync({
        provider: provider.slug,
        token_env: provider.token_env,
      });
      if (token) {
        const env = await setSessionEnv.mutateAsync({ provider: provider.slug, token });
        addExport(env.shell_export_block);
      } else {
        addExport(config.shell_export_block);
      }
      setToken("");
      setMessage(`Connected ${provider.label}${config.username ? ` as ${config.username}` : ""}.`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Could not connect provider");
    }
  }
  return (
    <Intro>
      <h2>Git provider</h2>
      <p>Choose a catalog provider. Existing CLI authentication is adopted when available.</p>
      <ChoiceGroup>
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
      </ChoiceGroup>
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
          {(message ?? providersQuery.error?.message) && (
            <p>{message ?? providersQuery.error?.message}</p>
          )}
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
      <ChoiceGroup>
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
      </ChoiceGroup>
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
      <ChoiceGroup>
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
      </ChoiceGroup>
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
      <ChoiceGroup>
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
      </ChoiceGroup>
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
      <ChoiceGroup>
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
      </ChoiceGroup>
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
      <ChoiceGroup>
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
      </ChoiceGroup>
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
      <ChoiceGroup>
        <Choice selected={!draft.useWorktrees} value="in-place" onSelect={selectWorktreePreference}>
          <strong>In-place branches</strong>
        </Choice>
        <Choice selected={draft.useWorktrees} value="worktree" onSelect={selectWorktreePreference}>
          <strong>Per-ticket worktrees</strong>
        </Choice>
      </ChoiceGroup>
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
  license: License,
  "execution-mode": ExecutionMode,
  "kanban-info": KanbanInfo,
  "model-server": ModelServer,
  "git-provider": GitProvider,
  "collection-source": CollectionSource,
  "task-field-config": TaskFieldConfig,
  "session-wrapper-choice": SessionWrapperChoice,
  "worktree-preference": WorktreePreference,
  "admin-password": AdminPassword,
  "tmux-onboarding": TmuxOnboarding,
  "vscode-setup": VSCodeSetup,
  "cmux-setup": CmuxSetup,
  "zellij-setup": ZellijSetup,
  "acceptance-criteria": AcceptanceCriteria,
  "startup-tickets": StartupTickets,
  "hosted-collections": HostedCollections,
  "execution-target": ExecutionTarget,
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
      step !== "admin-password" &&
      (step !== "execution-target" || draft.executionMode === "remote") &&
      (step !== "hosted-collections" || draft.preset === "custom") &&
      (!wrapperSteps.has(step) || step === wrapperStep[draft.wrapper]),
  );
}

/**
 * Steps a run may skip outright. The catalog gathers them just before `confirm`,
 * and the sidebar shows the skipped ones as dimmed, unnumbered rows so that
 * answering an earlier question never renumbers the steps already shown.
 */
export const OPTIONAL_STEPS = {
  "hosted-collections": "if a hosted collection source is chosen",
  "execution-target": "premium · if agents run on remote targets",
} as const satisfies Partial<Record<SetupStep, string>>;

type OptionalStep = keyof typeof OPTIONAL_STEPS;

/**
 * The sidebar's rows, in catalog order: every step of the walk numbered from 1,
 * plus a `number: null` placeholder for each optional step this draft skips.
 * Steps no run ever reaches (the admin password, the three unchosen wrappers)
 * are left out entirely.
 */
export function stepRows(steps: SetupStep[], draft: StepProps["draft"]): StepRow[] {
  const walk = new Set(visibleSteps(steps, draft));
  const rows: StepRow[] = [];
  let number = 0;
  for (const slug of steps) {
    if (walk.has(slug)) {
      number += 1;
      rows.push({ slug, number });
    } else if (slug in OPTIONAL_STEPS) {
      rows.push({ slug, number: null, hint: OPTIONAL_STEPS[slug as OptionalStep] });
    }
  }
  return rows;
}
