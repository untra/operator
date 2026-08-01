import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Alert, Button, Chip, IconButton, SelectInput, TextInput } from '../primitives';
import { SectionHeader } from '../SectionHeader';
import { postMessage, onMessage } from '../../vscodeApi';
import type {
  ExtensionToWebviewMessage,
  ModelServerKindEntry,
  ModelServerModelsResponse,
  DelegatorResponse,
} from '../../types/messages';

const BRAND_ICONS = ['anthropic', 'google', 'ollama', 'openrouter'];

interface ModelProvidersSectionProps {
  detectedTools: string[];
  apiReachable: boolean;
}

type ProbeMap = Record<string, ModelServerModelsResponse | undefined>;

function DismissableAlert({
  severity,
  onClose,
  children,
}: {
  severity: 'error' | 'success';
  onClose: () => void;
  children: React.ReactNode;
}) {
  return (
    <Alert severity={severity} className="op-mt-1 op-mb-1">
      <span className="op-row op-gap-1">
        <span>{children}</span>
        <IconButton aria-label="Close" onClick={onClose} style={{ padding: 0, color: 'inherit' }}>
          ✕
        </IconButton>
      </span>
    </Alert>
  );
}

export function ModelProvidersSection({ detectedTools, apiReachable }: ModelProvidersSectionProps) {
  const [kinds, setKinds] = useState<ModelServerKindEntry[]>([]);
  const [probes, setProbes] = useState<ProbeMap>({});
  const [delegators, setDelegators] = useState<DelegatorResponse[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const load = useCallback(() => {
    if (apiReachable) {postMessage({ type: 'getModelProviders' });}
  }, [apiReachable]);

  useEffect(() => {
    const cleanup = onMessage((msg: ExtensionToWebviewMessage) => {
      switch (msg.type) {
        case 'modelProvidersLoaded':
          setKinds(msg.kinds);
          setDelegators(msg.delegators);
          for (const k of msg.kinds) {postMessage({ type: 'probeProvider', slug: k.slug });}
          break;
        case 'providerProbed':
          setProbes((p) => ({ ...p, [msg.slug]: msg.result }));
          break;
        case 'delegatorCreated':
          setNotice(`Created delegator "${msg.name}".`);
          load();
          break;
        case 'modelProvidersError':
          setError(msg.error);
          break;
      }
    });
    return cleanup;
  }, [load]);

  useEffect(load, [load]);

  const firstParty = useMemo(() => kinds.filter((k) => k.category === 'first-party'), [kinds]);
  const gateways = useMemo(() => kinds.filter((k) => k.category === 'gateway'), [kinds]);

  return (
    <div className="op-mb-4">
      <SectionHeader id="section-model-providers" title="Model Providers" />
      <p className="op-body1 op-text-secondary op-mb-1">
        Connect model providers (distinct from the coding-agent CLIs) and create
        delegators from their live models. See the{' '}
        <a href="https://operator.untra.io/getting-started/model-servers/">
          model providers documentation
        </a>
        .
      </p>

      {!apiReachable && (
        <Alert severity="info" className="op-mt-1 op-mb-1">
          Start the operator API to connect providers and list models.
        </Alert>
      )}
      {error && (
        <DismissableAlert severity="error" onClose={() => setError(null)}>
          {error}
        </DismissableAlert>
      )}
      {notice && (
        <DismissableAlert severity="success" onClose={() => setNotice(null)}>
          {notice}
        </DismissableAlert>
      )}

      <ProviderGroup heading="First-party" kinds={firstParty} probes={probes} />
      <ProviderGroup heading="Gateways" kinds={gateways} probes={probes} />

      <CreateDelegatorForm kinds={kinds} probes={probes} detectedTools={detectedTools} />

      <p className="op-body2 op-text-secondary op-mt-2 op-mb-05">
        Delegators
      </p>
      {delegators.length === 0 ? (
        <p className="op-body2 op-text-secondary">
          No delegators yet.
        </p>
      ) : (
        <div className="op-col op-gap-05">
          {delegators.map((d) => (
            <p key={d.name} className="op-body2">
              <strong>{d.display_name ?? d.name}</strong>{' '}
              <span className="op-caption op-text-secondary">
                {d.llm_tool}:{d.model}
                {d.model_server ? ` @ ${d.model_server}` : ''}
              </span>
            </p>
          ))}
        </div>
      )}
    </div>
  );
}

function connection(probe: ModelServerModelsResponse | undefined): {
  color: 'success' | 'default' | 'warning';
  label: string;
} {
  if (probe === undefined) {return { color: 'warning', label: 'checking…' };}
  if (probe.reachable) {return { color: 'success', label: `connected · ${probe.models.length}` };}
  return { color: 'default', label: 'not connected' };
}

function ProviderGroup({
  heading,
  kinds,
  probes,
}: {
  heading: string;
  kinds: ModelServerKindEntry[];
  probes: ProbeMap;
}) {
  if (kinds.length === 0) {return null;}
  return (
    <div style={{ marginBottom: 12 }}>
      <p className="op-body2 op-text-secondary op-mb-05">
        {heading}
      </p>
      <div className="op-col" style={{ gap: 6 }}>
        {kinds.map((k) => {
          const probe = probes[k.slug];
          const conn = connection(probe);
          return (
            <div key={k.slug} className="op-row op-gap-1 op-wrap">
              {k.brand_icon && BRAND_ICONS.includes(k.brand_icon) && (
                <i className={`opi-${k.brand_icon}`} style={{ fontSize: '1rem', lineHeight: 1 }} />
              )}
              <span className="op-body2" style={{ fontWeight: 600, minWidth: '8rem' }}>
                {k.display_name}
              </span>
              <Chip label={conn.label} color={conn.color} variant="outlined" />
              {conn.label === 'not connected' && k.connectable && !k.is_builtin && (
                <Button size="small" onClick={() => postMessage({ type: 'connectProvider', slug: k.slug })}>
                  Connect
                </Button>
              )}
              {conn.label === 'not connected' && k.default_api_key_env && (
                <span className="op-caption op-text-secondary">
                  set {k.default_api_key_env}
                </span>
              )}
              {!k.connectable && (
                <a className="op-caption" href={k.setup_url} target="_blank" rel="noreferrer">
                  needs base_url
                </a>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function CreateDelegatorForm({
  kinds,
  probes,
  detectedTools,
}: {
  kinds: ModelServerKindEntry[];
  probes: ProbeMap;
  detectedTools: string[];
}) {
  const [tool, setTool] = useState('');
  const [provider, setProvider] = useState('');
  const [model, setModel] = useState('');
  const [name, setName] = useState('');

  useEffect(() => {
    if (!tool && detectedTools.length > 0) {setTool(detectedTools[0]);}
  }, [detectedTools, tool]);

  const probe = provider ? probes[provider] : undefined;
  const liveModels = probe?.reachable ? probe.models : [];

  const submit = () => {
    if (!tool || !provider || !model) {return;}
    postMessage({
      type: 'createDelegator',
      request: {
        name: name.trim() || `${tool}-${model}`,
        llm_tool: tool,
        model,
        display_name: null,
        model_properties: {},
        model_server: provider,
        launch_config: null,
        remote_agent: null,
      },
    });
    setName('');
    setModel('');
  };

  return (
    <div className="op-mt-2 op-mb-1">
      <p className="op-body2 op-text-secondary op-mb-1">
        Create delegator — pair a tool with a connected provider and a live model.
      </p>
      <div className="op-col" style={{ gap: 12, maxWidth: 420 }}>
        <SelectInput
          label="LLM tool"
          value={tool}
          onChange={(e) => setTool(e.target.value)}
        >
          {detectedTools.length === 0 && <option value="">(none detected)</option>}
          {detectedTools.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </SelectInput>

        <SelectInput
          label="Provider"
          value={provider}
          onChange={(e) => {
            setProvider(e.target.value);
            setModel('');
          }}
        >
          {kinds.map((k) => (
            <option key={k.slug} value={k.slug}>
              {k.display_name} {probes[k.slug]?.reachable ? '●' : '○'}
            </option>
          ))}
        </SelectInput>

        {liveModels.length > 0 ? (
          <SelectInput
            label="Model"
            value={model}
            onChange={(e) => setModel(e.target.value)}
          >
            {liveModels.map((m) => (
              <option key={m.id} value={m.id}>
                {m.display_name ?? m.id}
              </option>
            ))}
          </SelectInput>
        ) : (
          <TextInput
            label="Model"
            value={model}
            placeholder={provider ? 'model id (provider not connected)' : 'pick a provider first'}
            onChange={(e) => setModel(e.target.value)}
          />
        )}

        <TextInput
          label="Name (optional)"
          value={name}
          placeholder={tool && model ? `${tool}-${model}` : 'delegator name'}
          onChange={(e) => setName(e.target.value)}
        />

        <Button variant="outlined" size="small" onClick={submit} style={{ alignSelf: 'flex-start' }}>
          Create delegator
        </Button>
      </div>
    </div>
  );
}
