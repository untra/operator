// The Model Providers view — distinct from the LLM Tools (Coding Agents) page.
// Lists every supported provider (first-party vendors + gateways), shows each
// one's connection state (a live /models probe), and lets you create a delegator
// by picking a connected provider + one of its live models.

import { useCallback, useEffect, useMemo, useState } from 'react';
import { OperatorApi } from '../api-client';
import type {
  ModelServerKindEntry,
  ModelServerModelsResponse,
  Config,
  DelegatorResponse,
} from '../api-client';
import { useHost } from '../host';
import { CONCEPTS } from '../concepts';
import { PageHeader } from '../components/PageHeader';
import { BrandIcon } from '../components/BrandIcon';
import { ConceptIcon } from '../components/ConceptIcon';
import styles from './ModelProvidersPage.module.css';

const CONCEPT = CONCEPTS['model-servers'];

/** Live connection probe per provider slug. `undefined` = still loading. */
type ProbeMap = Record<string, ModelServerModelsResponse | undefined>;

export function ModelProvidersPage() {
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [kinds, setKinds] = useState<ModelServerKindEntry[]>([]);
  const [probes, setProbes] = useState<ProbeMap>({});
  const [detectedTools, setDetectedTools] = useState<string[]>([]);
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
    Promise.all([api.listProviderKinds(), api.getConfiguration()])
      .then(([catalog, config]: [ModelServerKindEntry[], Config]) => {
        if (cancelled) return;
        setKinds(catalog);
        setDetectedTools(config.llm_tools.detected.map((t) => t.name));
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

  if (loading) return <div className={styles.loading}>Loading model providers…</div>;

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
  if (probe === undefined) return { state: 'checking', text: 'checking…' };
  if (probe.reachable) return { state: 'connected', text: `${probe.models.length} models` };
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
  onConnect: (k: ModelServerKindEntry) => void;
}) {
  if (kinds.length === 0) return null;
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
  detectedTools: string[];
  onCreated: (name: string) => void;
  onError: (msg: string) => void;
}) {
  const [tool, setTool] = useState('');
  const [provider, setProvider] = useState('');
  const [model, setModel] = useState('');
  const [name, setName] = useState('');
  const [submitting, setSubmitting] = useState(false);

  // Default the tool once detection lands.
  useEffect(() => {
    if (!tool && detectedTools.length > 0) setTool(detectedTools[0]);
  }, [detectedTools, tool]);

  const probe = provider ? probes[provider] : undefined;
  const liveModels = probe?.reachable ? probe.models : [];

  const submit = async () => {
    if (!tool || !provider || !model) {
      onError('Pick a tool, a provider, and a model.');
      return;
    }
    setSubmitting(true);
    try {
      const delegatorName = name.trim() || `${tool}-${model}`;
      await api.createDelegator({
        name: delegatorName,
        llm_tool: tool,
        model,
        display_name: null,
        model_properties: {},
        model_server: provider,
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
          <select value={tool} onChange={(e) => setTool(e.target.value)} className={styles.select}>
            {detectedTools.length === 0 && <option value="">(none detected)</option>}
            {detectedTools.map((t) => (
              <option key={t} value={t}>
                {t}
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
            placeholder={tool && model ? `${tool}-${model}` : 'delegator name'}
            onChange={(e) => setName(e.target.value)}
          />
        </label>

        <button className={styles.submitBtn} onClick={submit} disabled={submitting}>
          {submitting ? 'Creating…' : 'Create delegator'}
        </button>
      </div>
    </section>
  );
}
