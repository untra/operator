import React, { useCallback, useEffect, useMemo, useState } from 'react';
import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Chip from '@mui/material/Chip';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select, { type SelectChangeEvent } from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';
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
    <Box sx={{ mb: 4 }}>
      <SectionHeader id="section-model-providers" title="Model Providers" />
      <Typography color="text.secondary" gutterBottom>
        Connect model providers (distinct from the coding-agent CLIs) and create
        delegators from their live models. See the{' '}
        <Link href="https://operator.untra.io/getting-started/model-servers/">
          model providers documentation
        </Link>
        .
      </Typography>

      {!apiReachable && (
        <Alert severity="info" sx={{ my: 1 }}>
          Start the operator API to connect providers and list models.
        </Alert>
      )}
      {error && (
        <Alert severity="error" sx={{ my: 1 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      {notice && (
        <Alert severity="success" sx={{ my: 1 }} onClose={() => setNotice(null)}>
          {notice}
        </Alert>
      )}

      <ProviderGroup heading="First-party" kinds={firstParty} probes={probes} />
      <ProviderGroup heading="Gateways" kinds={gateways} probes={probes} />

      <CreateDelegatorForm kinds={kinds} probes={probes} detectedTools={detectedTools} />

      <Typography variant="body2" color="text.secondary" sx={{ mt: 2, mb: 0.5 }}>
        Delegators
      </Typography>
      {delegators.length === 0 ? (
        <Typography variant="body2" color="text.secondary">
          No delegators yet.
        </Typography>
      ) : (
        <Stack spacing={0.5}>
          {delegators.map((d) => (
            <Typography key={d.name} variant="body2">
              <strong>{d.display_name ?? d.name}</strong>{' '}
              <Typography component="span" variant="caption" color="text.secondary">
                {d.llm_tool}:{d.model}
                {d.model_server ? ` @ ${d.model_server}` : ''}
              </Typography>
            </Typography>
          ))}
        </Stack>
      )}
    </Box>
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
    <Box sx={{ mb: 1.5 }}>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 0.5 }}>
        {heading}
      </Typography>
      <Stack spacing={0.75}>
        {kinds.map((k) => {
          const probe = probes[k.slug];
          const conn = connection(probe);
          return (
            <Stack key={k.slug} direction="row" spacing={1} alignItems="center" flexWrap="wrap" useFlexGap>
              {k.brand_icon && BRAND_ICONS.includes(k.brand_icon) && (
                <i className={`opi-${k.brand_icon}`} style={{ fontSize: '1rem', lineHeight: 1 }} />
              )}
              <Typography variant="body2" sx={{ fontWeight: 600, minWidth: '8rem' }}>
                {k.display_name}
              </Typography>
              <Chip size="small" label={conn.label} color={conn.color} variant="outlined" />
              {conn.label === 'not connected' && k.connectable && !k.is_builtin && (
                <Button size="small" onClick={() => postMessage({ type: 'connectProvider', slug: k.slug })}>
                  Connect
                </Button>
              )}
              {conn.label === 'not connected' && k.default_api_key_env && (
                <Typography variant="caption" color="text.secondary">
                  set {k.default_api_key_env}
                </Typography>
              )}
              {!k.connectable && (
                <Link variant="caption" href={k.setup_url} target="_blank" rel="noreferrer">
                  needs base_url
                </Link>
              )}
            </Stack>
          );
        })}
      </Stack>
    </Box>
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
    <Box sx={{ mt: 2, mb: 1 }}>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
        Create delegator — pair a tool with a connected provider and a live model.
      </Typography>
      <Stack spacing={1.5} sx={{ maxWidth: 420 }}>
        <FormControl size="small" fullWidth>
          <InputLabel>LLM tool</InputLabel>
          <Select label="LLM tool" value={tool} onChange={(e: SelectChangeEvent) => setTool(e.target.value)}>
            {detectedTools.length === 0 && <MenuItem value="">(none detected)</MenuItem>}
            {detectedTools.map((t) => (
              <MenuItem key={t} value={t}>
                {t}
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        <FormControl size="small" fullWidth>
          <InputLabel>Provider</InputLabel>
          <Select
            label="Provider"
            value={provider}
            onChange={(e: SelectChangeEvent) => {
              setProvider(e.target.value);
              setModel('');
            }}
          >
            {kinds.map((k) => (
              <MenuItem key={k.slug} value={k.slug}>
                {k.display_name} {probes[k.slug]?.reachable ? '●' : '○'}
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        {liveModels.length > 0 ? (
          <FormControl size="small" fullWidth>
            <InputLabel>Model</InputLabel>
            <Select label="Model" value={model} onChange={(e: SelectChangeEvent) => setModel(e.target.value)}>
              {liveModels.map((m) => (
                <MenuItem key={m.id} value={m.id}>
                  {m.display_name ?? m.id}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        ) : (
          <TextField
            size="small"
            fullWidth
            label="Model"
            value={model}
            placeholder={provider ? 'model id (provider not connected)' : 'pick a provider first'}
            onChange={(e) => setModel(e.target.value)}
          />
        )}

        <TextField
          size="small"
          fullWidth
          label="Name (optional)"
          value={name}
          placeholder={tool && model ? `${tool}-${model}` : 'delegator name'}
          onChange={(e) => setName(e.target.value)}
        />

        <Button variant="outlined" size="small" onClick={submit} sx={{ alignSelf: 'flex-start' }}>
          Create delegator
        </Button>
      </Stack>
    </Box>
  );
}
