// The Model Providers view - distinct from the LLM Tools (Coding Agents) page.
// Lists every supported provider (first-party vendors + gateways), shows each
// one's connection state (a live /models probe), and lets you create a delegator
// by picking a connected provider + one of its live models.

import { useCallback, useMemo, useState } from "react";
import type {
  ModelServerKindEntry,
  ModelServerModelsResponse,
  DelegatorResponse,
} from "../api-client";
import type { GitExecutionConfig } from "@operator/bindings/GitExecutionConfig";
import { useApiMutation, useApiQuery } from "../api";
import {
  createDelegatorMutation,
  createModelServerMutation,
  delegatorsQuery,
  llmToolsQuery,
  providerKindsQuery,
  providerModelsQuery,
  updateDelegatorMutation,
} from "../api/definitions";
import { CONCEPTS } from "../concepts";
import { PageHeader } from "../components/PageHeader";
import { BrandIcon } from "../components/BrandIcon";
import { ConceptIcon } from "../components/ConceptIcon";
import styles from "./ModelProvidersPage.module.css";

const CONCEPT = CONCEPTS["model-servers"];
const EMPTY_KINDS: ModelServerKindEntry[] = [];

/** A detected llm tool offered in the delegator form; unhealthy ones can't launch. */
type DetectedToolOption = { name: string; healthOk: boolean };

type GitSettingDraft = GitExecutionConfig["settings"][number] & { id: string };
type GitExecutionDraft = Omit<GitExecutionConfig, "settings"> & { settings: GitSettingDraft[] };

function createGitDraft(config: GitExecutionConfig | null): GitExecutionDraft | null {
  if (!config) {
    return null;
  }
  return {
    ...config,
    settings: config.settings.map((entry) => ({ ...entry, id: crypto.randomUUID() })),
  };
}

function serializeGitDraft(draft: GitExecutionDraft | null): GitExecutionConfig | null {
  if (!draft) {
    return null;
  }
  return {
    ...draft,
    settings: draft.settings.map(({ id: _id, ...entry }) => entry),
  };
}

export function ModelProvidersPage() {
  const kindsQuery = useApiQuery(providerKindsQuery());
  const toolsQuery = useApiQuery(llmToolsQuery());
  const delegatorsQueryResult = useApiQuery(delegatorsQuery());
  const connect = useApiMutation(createModelServerMutation);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const kinds = kindsQuery.data ?? EMPTY_KINDS;
  const detectedTools: DetectedToolOption[] = (toolsQuery.data?.tools ?? []).map((t) => ({
    name: t.name,
    healthOk: t.health_ok,
  }));
  const delegators = delegatorsQueryResult.data?.delegators ?? [];
  const loading = kindsQuery.isLoading || toolsQuery.isLoading || delegatorsQueryResult.isLoading;

  const connectGateway = useCallback(
    (kind: ModelServerKindEntry) => {
      setError(null);
      connect.mutate(
        {
          name: kind.slug,
          kind: kind.slug,
          base_url: kind.default_base_url ?? null,
          api_key_env: kind.default_api_key_env ?? null,
          extra_env: {},
          display_name: kind.display_name,
        },
        {
          onSuccess: () => {
            setNotice(`Declared "${kind.slug}". Re-probing…`);
          },
          onError: (cause) => {
            setError(cause.message);
          },
        },
      );
    },
    [connect],
  );

  const handleDelegatorCreated = useCallback((name: string) => {
    setNotice(`Created delegator "${name}".`);
  }, []);

  const firstParty = useMemo(() => kinds.filter((k) => k.category === "first-party"), [kinds]);
  const gateways = useMemo(() => kinds.filter((k) => k.category === "gateway"), [kinds]);

  if (loading) {
    return <div className={styles.loading}>Loading model providers…</div>;
  }

  return (
    <div className={styles.page}>
      <PageHeader
        title={CONCEPT.label}
        summary={CONCEPT.summary}
        docsUrl={CONCEPT.docsUrl}
        icon={CONCEPT.icon}
      />

      {(error ??
        kindsQuery.error?.message ??
        toolsQuery.error?.message ??
        delegatorsQueryResult.error?.message) && (
        <div className={styles.error}>
          {error ??
            kindsQuery.error?.message ??
            toolsQuery.error?.message ??
            delegatorsQueryResult.error?.message}
        </div>
      )}
      {notice && <div className={styles.notice}>{notice}</div>}

      <ProviderGroup
        heading="First-party"
        blurb="Vendors that produce their own models. Set the key env to connect."
        kinds={firstParty}
        onConnect={connectGateway}
      />
      <ProviderGroup
        heading="Gateways"
        blurb="Hosts and aggregators that front many models behind one endpoint."
        kinds={gateways}
        onConnect={connectGateway}
      />

      <CreateDelegatorForm
        kinds={kinds}
        detectedTools={detectedTools}
        onCreated={handleDelegatorCreated}
        onError={setError}
      />

      <section className={styles.group}>
        <h2 className={styles.groupHeading}>Delegators</h2>
        {delegators.length === 0 ? (
          <p className={styles.empty}>No delegators yet.</p>
        ) : (
          <ul className={styles.delegatorList}>
            {delegators.map((d) => (
              <li key={d.name} className={styles.delegatorRow}>
                <span className={styles.delegatorName}>{d.display_name ?? d.name}</span>
                <span className={styles.delegatorMeta}>
                  {d.llm_tool}:{d.model}
                  {d.model_server ? ` @ ${d.model_server}` : ""}
                </span>
                <DelegatorGitEditor key={d.name} delegator={d} />
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

function connectionLabel(probe: ModelServerModelsResponse | undefined): {
  state: "connected" | "disconnected" | "checking";
  text: string;
} {
  if (probe === undefined) {
    return { state: "checking", text: "checking…" };
  }
  if (probe.reachable) {
    return { state: "connected", text: `${probe.models.length} models` };
  }
  return { state: "disconnected", text: "not connected" };
}

function ProviderGroup({
  heading,
  blurb,
  kinds,
  onConnect,
}: {
  heading: string;
  blurb: string;
  kinds: ModelServerKindEntry[];
  onConnect: (k: ModelServerKindEntry) => void;
}) {
  if (kinds.length === 0) {
    return null;
  }
  return (
    <section className={styles.group}>
      <h2 className={styles.groupHeading}>{heading}</h2>
      <p className={styles.groupBlurb}>{blurb}</p>
      <ul className={styles.providerList}>
        {kinds.map((k) => (
          <ProviderRow key={k.slug} kind={k} onConnect={onConnect} />
        ))}
      </ul>
    </section>
  );
}

function ProviderRow({
  kind,
  onConnect,
}: {
  kind: ModelServerKindEntry;
  onConnect: (k: ModelServerKindEntry) => void;
}) {
  const query = useApiQuery(providerModelsQuery(kind.slug));
  const probe = query.error
    ? { server: kind.slug, reachable: false, models: [], error: "probe failed" }
    : (query.data ?? undefined);
  const conn = connectionLabel(probe);
  return (
    <li className={styles.providerRow}>
      <span className={styles.providerIcon}>
        {kind.brand_icon ? <BrandIcon name={kind.brand_icon} /> : <ConceptIcon name={kind.icon} />}
      </span>
      <span className={styles.providerName}>{kind.display_name}</span>
      <span className={styles.providerDesc}>{kind.description}</span>
      <span className={`${styles.dot} ${styles[conn.state]}`} />
      <span className={styles.connText}>{conn.text}</span>
      {conn.state === "disconnected" && kind.connectable && !kind.is_builtin && (
        <button className={styles.connectBtn} onClick={() => onConnect(kind)}>
          Connect
        </button>
      )}
      {conn.state === "disconnected" && kind.default_api_key_env && (
        <span className={styles.hint}>set {kind.default_api_key_env}</span>
      )}
      {!kind.connectable && (
        <a className={styles.hint} href={kind.setup_url} target="_blank" rel="noreferrer">
          needs base_url
        </a>
      )}
    </li>
  );
}

function CreateDelegatorForm({
  kinds,
  detectedTools,
  onCreated,
  onError,
}: {
  kinds: ModelServerKindEntry[];
  detectedTools: DetectedToolOption[];
  onCreated: (name: string) => void;
  onError: (msg: string) => void;
}) {
  const create = useApiMutation(createDelegatorMutation);
  const [tool, setTool] = useState("");
  const [provider, setProvider] = useState("");
  const [model, setModel] = useState("");
  const [name, setName] = useState("");
  const [git, setGit] = useState<GitExecutionDraft | null>(null);
  const selectedModels = useApiQuery(providerModelsQuery(provider), { enabled: Boolean(provider) });

  const preferredTool = detectedTools.find((candidate) => candidate.healthOk) ?? detectedTools[0];
  const selectedTool = tool || preferredTool?.name || "";

  const liveModels =
    !selectedModels.error && selectedModels.data?.reachable ? selectedModels.data.models : [];

  const submit = () => {
    if (!selectedTool || !provider || !model) {
      onError("Pick a tool, a provider, and a model.");
      return;
    }
    const delegatorName = name.trim() || `${selectedTool}-${model}`;
    create.mutate(
      {
        name: delegatorName,
        llm_tool: selectedTool,
        model,
        display_name: null,
        model_properties: {},
        model_server: provider,
        git: serializeGitDraft(git),
        launch_config: null,
        remote_agent: null,
      },
      {
        onSuccess: () => {
          setName("");
          setModel("");
          onCreated(delegatorName);
        },
        onError: (cause) => {
          onError(cause.message);
        },
      },
    );
  };

  return (
    <section className={styles.group}>
      <h2 className={styles.groupHeading}>Create delegator</h2>
      <p className={styles.groupBlurb}>
        Pair an llm tool with a connected provider and one of its live models.
      </p>
      <div className={styles.form}>
        <label className={styles.field}>
          <span className={styles.fieldLabel}>LLM tool</span>
          <select
            value={selectedTool}
            onChange={(e) => setTool(e.target.value)}
            className={styles.select}
          >
            {detectedTools.length === 0 && <option value="">(none detected)</option>}
            {detectedTools.map((t) => (
              <option key={t.name} value={t.name}>
                {t.healthOk ? t.name : `${t.name} (health check failed)`}
              </option>
            ))}
          </select>
        </label>

        <label className={styles.field}>
          <span className={styles.fieldLabel}>Provider</span>
          <select
            value={provider}
            onChange={(e) => {
              setProvider(e.target.value);
              setModel("");
            }}
            className={styles.select}
          >
            <option value="">Select…</option>
            {kinds.map((k) => (
              <option key={k.slug} value={k.slug}>
                {k.display_name}
              </option>
            ))}
          </select>
        </label>

        <label className={styles.field}>
          <span className={styles.fieldLabel}>Model</span>
          {liveModels.length > 0 ? (
            <select
              value={model}
              onChange={(e) => setModel(e.target.value)}
              className={styles.select}
            >
              <option value="">Select…</option>
              {liveModels.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.display_name ?? m.id}
                </option>
              ))}
            </select>
          ) : (
            <input
              className={styles.input}
              value={model}
              placeholder={provider ? "model id (provider not connected)" : "pick a provider first"}
              onChange={(e) => setModel(e.target.value)}
            />
          )}
        </label>

        <label className={styles.field}>
          <span className={styles.fieldLabel}>Name (optional)</span>
          <input
            className={styles.input}
            value={name}
            placeholder={selectedTool && model ? `${selectedTool}-${model}` : "delegator name"}
            onChange={(e) => setName(e.target.value)}
          />
        </label>

        <GitFields value={git} onChange={setGit} disabled={create.isPending} />
        <button className={styles.submitBtn} onClick={submit} disabled={create.isPending}>
          {create.isPending ? "Creating…" : "Create delegator"}
        </button>
      </div>
    </section>
  );
}

function GitFields({
  value,
  onChange,
  disabled,
}: {
  value: GitExecutionDraft | null;
  onChange: (value: GitExecutionDraft | null) => void;
  disabled: boolean;
}) {
  const config: GitExecutionDraft = value ?? { identity: null, credentials: null, settings: [] };
  const identity = config.identity ?? { name: "", email: "" };
  const credentials = config.credentials ?? { repository_url: "", username: "", token_env: "" };
  return (
    <fieldset disabled={disabled}>
      <legend>Git identity and credentials</legend>
      <label>
        <input
          type="checkbox"
          checked={value !== null}
          onChange={(e) => onChange(e.target.checked ? config : null)}
        />
        Customize Git for this delegator
      </label>
      {value && (
        <>
          <label>
            <input
              type="checkbox"
              checked={config.identity !== null}
              onChange={(e) =>
                onChange({ ...config, identity: e.target.checked ? identity : null })
              }
            />
            Set commit identity
          </label>
          {config.identity && (
            <>
              <label className={styles.field}>
                Commit name
                <input
                  className={styles.input}
                  value={identity.name}
                  onChange={(e) =>
                    onChange({ ...config, identity: { ...identity, name: e.target.value } })
                  }
                />
              </label>
              <label className={styles.field}>
                Commit email
                <input
                  className={styles.input}
                  value={identity.email}
                  onChange={(e) =>
                    onChange({ ...config, identity: { ...identity, email: e.target.value } })
                  }
                />
              </label>
              <p>
                Templates support {"{ticket_id}"}, {"{project}"}, and {"{ticket_type}"}.
              </p>
            </>
          )}
          <label>
            <input
              type="checkbox"
              checked={config.credentials !== null}
              onChange={(e) =>
                onChange({ ...config, credentials: e.target.checked ? credentials : null })
              }
            />
            Supply HTTPS credentials
          </label>
          {config.credentials && (
            <>
              <label className={styles.field}>
                HTTPS repository URL
                <input
                  className={styles.input}
                  value={credentials.repository_url}
                  onChange={(e) =>
                    onChange({
                      ...config,
                      credentials: { ...credentials, repository_url: e.target.value },
                    })
                  }
                />
              </label>
              <label className={styles.field}>
                Git username
                <input
                  className={styles.input}
                  value={credentials.username}
                  onChange={(e) =>
                    onChange({
                      ...config,
                      credentials: { ...credentials, username: e.target.value },
                    })
                  }
                />
              </label>
              <label className={styles.field}>
                Token environment variable
                <input
                  className={styles.input}
                  value={credentials.token_env}
                  onChange={(e) =>
                    onChange({
                      ...config,
                      credentials: { ...credentials, token_env: e.target.value },
                    })
                  }
                />
              </label>
              <p>Enter the variable name configured on Operator, such as AGENT_GIT_TOKEN.</p>
            </>
          )}
          {config.settings.map((entry, index) => (
            <div key={entry.id}>
              <label>
                Git setting
                <input
                  className={styles.input}
                  value={entry.key}
                  onChange={(e) =>
                    onChange({
                      ...config,
                      settings: config.settings.map((v, i) =>
                        i === index ? { ...v, key: e.target.value } : v,
                      ),
                    })
                  }
                />
              </label>
              <label>
                Value
                <input
                  className={styles.input}
                  value={entry.value}
                  onChange={(e) =>
                    onChange({
                      ...config,
                      settings: config.settings.map((v, i) =>
                        i === index ? { ...v, value: e.target.value } : v,
                      ),
                    })
                  }
                />
              </label>
              <button
                type="button"
                onClick={() =>
                  onChange({ ...config, settings: config.settings.filter((_, i) => i !== index) })
                }
              >
                Remove setting
              </button>
            </div>
          ))}
          <button
            type="button"
            onClick={() =>
              onChange({
                ...config,
                settings: [...config.settings, { id: crypto.randomUUID(), key: "", value: "" }],
              })
            }
          >
            Add Git setting
          </button>
        </>
      )}
    </fieldset>
  );
}

function DelegatorGitEditor({ delegator }: { delegator: DelegatorResponse }) {
  const update = useApiMutation(updateDelegatorMutation);
  const [git, setGit] = useState<GitExecutionDraft | null>(() =>
    createGitDraft(delegator.git ?? null),
  );
  const [error, setError] = useState("");
  const save = () => {
    setError("");
    update.mutate(
      {
        name: delegator.name,
        request: {
          name: delegator.name,
          llm_tool: delegator.llm_tool,
          model: delegator.model,
          display_name: delegator.display_name ?? null,
          model_properties: delegator.model_properties,
          model_server: delegator.model_server ?? null,
          launch_config: delegator.launch_config ?? null,
          remote_agent: delegator.remote_agent ?? null,
          git: serializeGitDraft(git),
        },
      },
      {
        onSuccess: (saved) => {
          setGit(createGitDraft(saved.git ?? null));
        },
        onError: (cause) => {
          setError(cause.message);
        },
      },
    );
  };
  return (
    <details>
      <summary>Git settings</summary>
      <GitFields value={git} onChange={setGit} disabled={update.isPending} />
      {error && <p role="alert">{error}</p>}
      <button type="button" onClick={save} disabled={update.isPending}>
        {update.isPending ? "Saving…" : "Save Git settings"}
      </button>
    </details>
  );
}
