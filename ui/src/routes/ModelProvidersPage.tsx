// The Model Providers view — distinct from the LLM Tools (Coding Agents) page.
// Lists every supported provider (first-party vendors + gateways), shows each
// one's connection state (a live /models probe), and lets you create a delegator
// by picking a connected provider + one of its live models.

import { useCallback, useEffect, useMemo, useState } from 'react';
import { OperatorApi } from '../api-client';
import type {
  ModelServerKindEntry,
  ModelServerModelsResponse,
  LlmToolsResponse,
  DelegatorResponse,
} from '../api-client';
import type { GitExecutionConfig } from '@operator/bindings/GitExecutionConfig';
import { useHost } from '../host';
import { CONCEPTS } from '../concepts';
import { PageHeader } from '../components/PageHeader';
import { BrandIcon } from '../components/BrandIcon';
import { ConceptIcon } from '../components/ConceptIcon';
import styles from './ModelProvidersPage.module.css';

const CONCEPT = CONCEPTS['model-servers'];

/** Live connection probe per provider slug. `undefined` = still loading. */
type ProbeMap = Record<string, ModelServerModelsResponse | undefined>;

/** A detected llm tool offered in the delegator form; unhealthy ones can't launch. */
type DetectedToolOption = { name: string; healthOk: boolean };

type GitSettingDraft = GitExecutionConfig['settings'][number] & { id: string };
type GitExecutionDraft = Omit<GitExecutionConfig, 'settings'> & { settings: GitSettingDraft[] };

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
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [kinds, setKinds] = useState<ModelServerKindEntry[]>([]);
  const [probes, setProbes] = useState<ProbeMap>({});
  const [detectedTools, setDetectedTools] = useState<DetectedToolOption[]>([]);
  const [delegators, setDelegators] = useState<DelegatorResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refreshDelegators = useCallback(() => {
    api
      .listDelegators()
      .then((r) => setDelegators(r.delegators))
      .catch(() => {/* non-fatal */});
  }, [api]);

  // Load the catalog + detected tools, then probe each provider for connection.
  useEffect(() => {
    let cancelled = false;
    Promise.all([api.listProviderKinds(), api.listLlmTools()])
      .then(([catalog, tools]: [ModelServerKindEntry[], LlmToolsResponse]) => {
        if (!cancelled) {
          setKinds(catalog);
          setDetectedTools(
            tools.tools.map((t) => ({ name: t.name, healthOk: t.health_ok })),
          );
          // Probe each provider concurrently; fill the map as results land.
          for (const k of catalog) {
            api
              .providerModels(k.slug)
              .then((r) => !cancelled && setProbes((p) => ({ ...p, [k.slug]: r })))
              .catch(
                () =>
                  !cancelled &&
                  setProbes((p) => ({
                    ...p,
                    [k.slug]: { server: k.slug, reachable: false, models: [], error: 'probe failed' },
                  })),
            );
          }
        }
        return undefined;
      })
      .catch((e) => !cancelled && setError(e instanceof Error ? e.message : 'Failed to load'))
      .finally(() => !cancelled && setLoading(false));
    return () => {
      cancelled = true;
    };
  }, [api]);

  useEffect(refreshDelegators, [refreshDelegators]);

  const connectGateway = async (kind: ModelServerKindEntry) => {
    setError(null);
    try {
      await api.createModelServer({
        name: kind.slug,
        kind: kind.slug,
        base_url: kind.default_base_url ?? null,
        api_key_env: kind.default_api_key_env ?? null,
        extra_env: {},
        display_name: kind.display_name,
      });
      setNotice(`Declared "${kind.slug}". Re-probing…`);
      const r = await api.providerModels(kind.slug);
      setProbes((p) => ({ ...p, [kind.slug]: r }));
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to connect provider');
    }
  };

  const firstParty = useMemo(() => kinds.filter((k) => k.category === 'first-party'), [kinds]);
  const gateways = useMemo(() => kinds.filter((k) => k.category === 'gateway'), [kinds]);

  if (loading) {return <div className={styles.loading}>Loading model providers…</div>;}

  return (
    <div className={styles.page}>
      <PageHeader
        title={CONCEPT.label}
        summary={CONCEPT.summary}
        docsUrl={CONCEPT.docsUrl}
        icon={CONCEPT.icon}
      />

      {error && <div className={styles.error}>{error}</div>}
      {notice && <div className={styles.notice}>{notice}</div>}

      <ProviderGroup
        heading="First-party"
        blurb="Vendors that produce their own models. Set the key env to connect."
        kinds={firstParty}
        probes={probes}
        onConnect={connectGateway}
      />
      <ProviderGroup
        heading="Gateways"
        blurb="Hosts and aggregators that front many models behind one endpoint."
        kinds={gateways}
        probes={probes}
        onConnect={connectGateway}
      />

      <CreateDelegatorForm
        api={api}
        kinds={kinds}
        probes={probes}
        detectedTools={detectedTools}
        onCreated={(name) => {
          setNotice(`Created delegator "${name}".`);
          refreshDelegators();
        }}
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
                  {d.model_server ? ` @ ${d.model_server}` : ''}
                </span>
                <DelegatorGitEditor
                  key={`${d.name}:${JSON.stringify(d.git)}`}
                  api={api}
                  delegator={d}
                  onSaved={refreshDelegators}
                />
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

function connectionLabel(probe: ModelServerModelsResponse | undefined): {
  state: 'connected' | 'disconnected' | 'checking';
  text: string;
} {
  if (probe === undefined) {return { state: 'checking', text: 'checking…' };}
  if (probe.reachable) {return { state: 'connected', text: `${probe.models.length} models` };}
  return { state: 'disconnected', text: 'not connected' };
}

function ProviderGroup({
  heading,
  blurb,
  kinds,
  probes,
  onConnect,
}: {
  heading: string;
  blurb: string;
  kinds: ModelServerKindEntry[];
  probes: ProbeMap;
  onConnect: (k: ModelServerKindEntry) => Promise<void>;
}) {
  if (kinds.length === 0) {return null;}
  return (
    <section className={styles.group}>
      <h2 className={styles.groupHeading}>{heading}</h2>
      <p className={styles.groupBlurb}>{blurb}</p>
      <ul className={styles.providerList}>
        {kinds.map((k) => {
          const probe = probes[k.slug];
          const conn = connectionLabel(probe);
          return (
            <li key={k.slug} className={styles.providerRow}>
              <span className={styles.providerIcon}>
                {k.brand_icon ? <BrandIcon name={k.brand_icon} /> : <ConceptIcon name={k.icon} />}
              </span>
              <span className={styles.providerName}>{k.display_name}</span>
              <span className={styles.providerDesc}>{k.description}</span>
              <span className={`${styles.dot} ${styles[conn.state]}`} />
              <span className={styles.connText}>{conn.text}</span>
              {conn.state === 'disconnected' && k.connectable && !k.is_builtin && (
                <button className={styles.connectBtn} onClick={() => onConnect(k)}>
                  Connect
                </button>
              )}
              {conn.state === 'disconnected' && k.default_api_key_env && (
                <span className={styles.hint}>set {k.default_api_key_env}</span>
              )}
              {!k.connectable && (
                <a className={styles.hint} href={k.setup_url} target="_blank" rel="noreferrer">
                  needs base_url
                </a>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}

function CreateDelegatorForm({
  api,
  kinds,
  probes,
  detectedTools,
  onCreated,
  onError,
}: {
  api: OperatorApi;
  kinds: ModelServerKindEntry[];
  probes: ProbeMap;
  detectedTools: DetectedToolOption[];
  onCreated: (name: string) => void;
  onError: (msg: string) => void;
}) {
  const [tool, setTool] = useState('');
  const [provider, setProvider] = useState('');
  const [model, setModel] = useState('');
  const [name, setName] = useState('');
  const [git, setGit] = useState<GitExecutionDraft | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const preferredTool = detectedTools.find((candidate) => candidate.healthOk) ?? detectedTools[0];
  const selectedTool = tool || preferredTool?.name || '';

  const probe = provider ? probes[provider] : undefined;
  const liveModels = probe?.reachable ? probe.models : [];

  const submit = async () => {
    if (!selectedTool || !provider || !model) {
      onError('Pick a tool, a provider, and a model.');
      return;
    }
    setSubmitting(true);
    try {
      const delegatorName = name.trim() || `${selectedTool}-${model}`;
      await api.createDelegator({
        name: delegatorName,
        llm_tool: selectedTool,
        model,
        display_name: null,
        model_properties: {},
        model_server: provider,
        git: serializeGitDraft(git),
        launch_config: null,
        remote_agent: null,
      });
      setName('');
      setModel('');
      onCreated(delegatorName);
    } catch (e) {
      onError(e instanceof Error ? e.message : 'Failed to create delegator');
    } finally {
      setSubmitting(false);
    }
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
          <select value={selectedTool} onChange={(e) => setTool(e.target.value)} className={styles.select}>
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
              setModel('');
            }}
            className={styles.select}
          >
            <option value="">Select…</option>
            {kinds.map((k) => {
              const connected = probes[k.slug]?.reachable;
              return (
                <option key={k.slug} value={k.slug}>
                  {k.display_name}
                  {connected ? ' ●' : ' ○'}
                </option>
              );
            })}
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
            // Provider not connected (or no models) — fall back to free-text so
            // the form still works offline / pre-auth.
            <input
              className={styles.input}
              value={model}
              placeholder={provider ? 'model id (provider not connected)' : 'pick a provider first'}
              onChange={(e) => setModel(e.target.value)}
            />
          )}
        </label>

        <label className={styles.field}>
          <span className={styles.fieldLabel}>Name (optional)</span>
          <input
            className={styles.input}
            value={name}
            placeholder={selectedTool && model ? `${selectedTool}-${model}` : 'delegator name'}
            onChange={(e) => setName(e.target.value)}
          />
        </label>

        <GitFields value={git} onChange={setGit} disabled={submitting} />
        <button className={styles.submitBtn} onClick={submit} disabled={submitting}>
          {submitting ? 'Creating…' : 'Create delegator'}
        </button>
      </div>
    </section>
  );
}

function GitFields({ value, onChange, disabled }: {
  value: GitExecutionDraft | null;
  onChange: (value: GitExecutionDraft | null) => void;
  disabled: boolean;
}) {
  const config: GitExecutionDraft = value ?? { identity: null, credentials: null, settings: [] };
  const identity = config.identity ?? { name: '', email: '' };
  const credentials = config.credentials ?? { repository_url: '', username: '', token_env: '' };
  return <fieldset disabled={disabled}>
    <legend>Git identity and credentials</legend>
    <label><input type="checkbox" checked={value !== null} onChange={e => onChange(e.target.checked ? config : null)} />Customize Git for this delegator</label>
    {value && <>
      <label><input type="checkbox" checked={config.identity !== null} onChange={e => onChange({ ...config, identity: e.target.checked ? identity : null })} />Set commit identity</label>
      {config.identity && <>
        <label className={styles.field}>Commit name<input className={styles.input} value={identity.name} onChange={e => onChange({ ...config, identity: { ...identity, name: e.target.value } })} /></label>
        <label className={styles.field}>Commit email<input className={styles.input} value={identity.email} onChange={e => onChange({ ...config, identity: { ...identity, email: e.target.value } })} /></label>
        <p>Templates support {'{ticket_id}'}, {'{project}'}, and {'{ticket_type}'}.</p>
      </>}
      <label><input type="checkbox" checked={config.credentials !== null} onChange={e => onChange({ ...config, credentials: e.target.checked ? credentials : null })} />Supply HTTPS credentials</label>
      {config.credentials && <>
        <label className={styles.field}>HTTPS repository URL<input className={styles.input} value={credentials.repository_url} onChange={e => onChange({ ...config, credentials: { ...credentials, repository_url: e.target.value } })} /></label>
        <label className={styles.field}>Git username<input className={styles.input} value={credentials.username} onChange={e => onChange({ ...config, credentials: { ...credentials, username: e.target.value } })} /></label>
        <label className={styles.field}>Token environment variable<input className={styles.input} value={credentials.token_env} onChange={e => onChange({ ...config, credentials: { ...credentials, token_env: e.target.value } })} /></label>
        <p>Enter the variable name configured on Operator, such as AGENT_GIT_TOKEN.</p>
      </>}
      {config.settings.map((entry, index) => <div key={entry.id}>
        <label>Git setting<input className={styles.input} value={entry.key} onChange={e => onChange({ ...config, settings: config.settings.map((v, i) => i === index ? { ...v, key: e.target.value } : v) })} /></label>
        <label>Value<input className={styles.input} value={entry.value} onChange={e => onChange({ ...config, settings: config.settings.map((v, i) => i === index ? { ...v, value: e.target.value } : v) })} /></label>
        <button type="button" onClick={() => onChange({ ...config, settings: config.settings.filter((_, i) => i !== index) })}>Remove setting</button>
      </div>)}
      <button type="button" onClick={() => onChange({ ...config, settings: [...config.settings, { id: crypto.randomUUID(), key: '', value: '' }] })}>Add Git setting</button>
    </>}
  </fieldset>;
}

function DelegatorGitEditor({ api, delegator, onSaved }: { api: OperatorApi; delegator: DelegatorResponse; onSaved: () => void }) {
  const [git, setGit] = useState<GitExecutionDraft | null>(() => createGitDraft(delegator.git ?? null));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const save = async () => {
    setBusy(true);
    setError('');
    try {
      await api.updateDelegator(delegator.name, {
        name: delegator.name, llm_tool: delegator.llm_tool, model: delegator.model,
        display_name: delegator.display_name ?? null, model_properties: delegator.model_properties,
        model_server: delegator.model_server ?? null, launch_config: delegator.launch_config ?? null,
        remote_agent: delegator.remote_agent ?? null, git: serializeGitDraft(git),
      });
      onSaved();
    } catch (e) { setError(e instanceof Error ? e.message : 'Failed to save Git settings'); }
    finally { setBusy(false); }
  };
  return <details><summary>Git settings</summary>
    <GitFields value={git} onChange={setGit} disabled={busy} />
    {error && <p role="alert">{error}</p>}
    <button type="button" onClick={save} disabled={busy}>{busy ? 'Saving…' : 'Save Git settings'}</button>
  </details>;
}
