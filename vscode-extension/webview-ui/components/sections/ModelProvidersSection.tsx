import type { ModelServerResponse } from "../../../src/generated/ModelServerResponse";
import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Alert, Button, Chip, IconButton, SelectInput, TextInput } from "../primitives";
import { SectionHeader } from "../SectionHeader";
import { postMessage, onMessage } from "../../vscodeApi";
import type {
  ExtensionToWebviewMessage,
  ModelServerKindEntry,
  ModelServerModelsResponse,
  DelegatorResponse,
  NavigationPrefill,
} from "../../types/messages";

const BRAND_ICONS = new Set(["anthropic", "google", "ollama", "openrouter", "xai", "lmstudio"]);

interface ModelProvidersSectionProps {
  detectedTools: string[];
  apiReachable: boolean;
  delegatorPrefill?: NavigationPrefill;
}

type ProbeMap = Record<string, ModelServerModelsResponse | undefined>;

function connectProvider(event: React.MouseEvent<HTMLButtonElement>): void {
  const slug = event.currentTarget.dataset.providerSlug;
  if (slug) {
    postMessage({ type: "connectProvider", slug });
  }
}

function DismissableAlert({
  severity,
  onClose,
  children,
}: {
  severity: "error" | "success";
  onClose: () => void;
  children: React.ReactNode;
}) {
  return (
    <Alert severity={severity} className="op-mt-1 op-mb-1">
      <span className="op-row op-gap-1">
        <span>{children}</span>
        <IconButton aria-label="Close" onClick={onClose} className="op-alert-close">
          ✕
        </IconButton>
      </span>
    </Alert>
  );
}

export function ModelProvidersSection({
  detectedTools,
  apiReachable,
  delegatorPrefill,
}: ModelProvidersSectionProps) {
  const [kinds, setKinds] = useState<ModelServerKindEntry[]>([]);
  const [servers, setServers] = useState<ModelServerResponse[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [probes, setProbes] = useState<ProbeMap>({});
  const [delegators, setDelegators] = useState<DelegatorResponse[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const dismissError = useCallback(() => setError(null), [setError]);
  const dismissNotice = useCallback(() => setNotice(null), [setNotice]);

  const load = useCallback(() => {
    if (apiReachable) {
      postMessage({ type: "getModelProviders" });
    }
  }, [apiReachable]);

  useEffect(() => {
    const cleanup = onMessage((msg: ExtensionToWebviewMessage) => {
      switch (msg.type) {
        case "modelProvidersLoaded":
          setKinds(msg.kinds);
          setDelegators(msg.delegators);
          setLoaded(true);
          setError(null);
          setServers(msg.servers);
          setProbes({});
          for (const k of msg.servers) {
            postMessage({ type: "probeProvider", slug: k.name });
          }
          break;
        case "providerProbed":
          setProbes((p) => ({ ...p, [msg.slug]: msg.result }));
          break;
        case "delegatorCreated":
          setNotice(`Created delegator "${msg.name}".`);
          load();
          break;
        case "modelProvidersError":
        case "delegatorCreateError":
          setError(msg.error);
          break;
        case "apiHealthResult":
        case "assessTicketCreated":
        case "assessTicketError":
        case "browseResult":
        case "collectionActivated":
        case "collectionsError":
        case "collectionsLoaded":
        case "configError":
        case "configLoaded":
        case "configUpdated":
        case "externalIssueTypesError":
        case "externalIssueTypesLoaded":
        case "issueTypeCreated":
        case "issueTypeDeleted":
        case "issueTypeError":
        case "issueTypeLoaded":
        case "issueTypeUpdated":
        case "issueTypesLoaded":
        case "jiraValidationResult":
        case "kanbanStatusesError":
        case "kanbanStatusesLoaded":
        case "linearValidationResult":
        case "llmToolsDetected":
        case "navigateTo":
        case "projectsError":
        case "projectsLoaded":
          break;
      }
    });
    return cleanup;
  }, [load]);

  useEffect(load, [load]);

  const firstParty = useMemo(() => kinds.filter((k) => k.category === "first-party"), [kinds]);
  const gateways = useMemo(() => kinds.filter((k) => k.category === "gateway"), [kinds]);

  return (
    <div className="op-mb-4">
      <SectionHeader id="section-model-providers" title="Model Providers" />
      <p className="op-body1 op-text-secondary op-mb-1">
        Connect model providers (distinct from the coding-agent CLIs) and create delegators from
        their live models. See the{" "}
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
        <DismissableAlert severity="error" onClose={dismissError}>
          {error}
        </DismissableAlert>
      )}
      {notice && (
        <DismissableAlert severity="success" onClose={dismissNotice}>
          {notice}
        </DismissableAlert>
      )}

      {apiReachable && (
        <>
          {!loaded && !error && <p>Loading model providers…</p>}
          <ProviderGroup
            heading="First-party"
            kinds={firstParty}
            servers={servers}
            probes={probes}
          />
          <ProviderGroup heading="Gateways" kinds={gateways} servers={servers} probes={probes} />

          <CreateDelegatorForm
            servers={servers}
            probes={probes}
            detectedTools={detectedTools}
            prefill={delegatorPrefill}
            onInvalid={setError}
          />

          <p className="op-body2 op-text-secondary op-mt-2 op-mb-05">Delegators</p>
          {loaded && delegators.length === 0 ? (
            <p className="op-body2 op-text-secondary">No delegators yet.</p>
          ) : (
            <div className="op-col op-gap-05">
              {delegators.map((d) => (
                <p key={d.name} className="op-body2">
                  <strong>{d.display_name ?? d.name}</strong>{" "}
                  <span className="op-caption op-text-secondary">
                    {d.llm_tool}:{d.model}
                    {d.model_server ? ` @ ${d.model_server}` : ""}
                  </span>
                </p>
              ))}
            </div>
          )}
        </>
      )}
    </div>
  );
}

function connection(probe: ModelServerModelsResponse | undefined): {
  color: "success" | "default" | "warning";
  label: string;
} {
  if (probe === undefined) {
    return { color: "warning", label: "checking…" };
  }
  if (probe.reachable) {
    return { color: "success", label: `connected · ${probe.models.length}` };
  }
  return {
    color: "warning",
    label:
      probe.status === "missing_key"
        ? "API key required"
        : probe.status === "unauthorized"
          ? "Authorization failed"
          : probe.error || "Unreachable",
  };
}

function ProviderGroup({
  heading,
  servers,
  kinds,
  probes,
}: {
  heading: string;
  servers: ModelServerResponse[];
  kinds: ModelServerKindEntry[];
  probes: ProbeMap;
}) {
  if (kinds.length === 0) {
    return null;
  }
  return (
    <div className="op-provider-group">
      <p className="op-body2 op-text-secondary op-mb-05">{heading}</p>
      <div className="op-col op-provider-list">
        {kinds.map((k) => {
          const instances = servers.filter((server) => server.kind === k.slug);
          return (
            <div key={k.slug} className="op-row op-gap-1 op-wrap">
              {k.brand_icon && BRAND_ICONS.has(k.brand_icon) && (
                <i className={`op-brand-glyph opi-${k.brand_icon}`} />
              )}
              <span className="op-body2 op-provider-label">{k.display_name}</span>
              {instances.map((instance) => {
                const state = connection(probes[instance.name]);
                return (
                  <Chip
                    key={instance.name}
                    label={`${instance.name} · ${state.label}`}
                    color={state.color}
                    variant="outlined"
                  />
                );
              })}
              {instances.length === 0 && k.connectable && !k.is_builtin && (
                <Button size="small" data-provider-slug={k.slug} onClick={connectProvider}>
                  Connect
                </Button>
              )}
              {instances.some((server) => probes[server.name]?.status === "missing_key") &&
                k.default_api_key_env && (
                  <span className="op-caption op-text-secondary">set {k.default_api_key_env}</span>
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
  servers,
  probes,
  detectedTools,
  prefill,
  onInvalid,
}: {
  servers: ModelServerResponse[];
  probes: ProbeMap;
  detectedTools: string[];
  prefill?: NavigationPrefill;
  onInvalid: (message: string) => void;
}) {
  const [tool, setTool] = useState(prefill?.tool ?? "");
  const [provider, setProvider] = useState("");
  const [model, setModel] = useState(prefill?.model ?? "");
  const [name, setName] = useState("");
  const [manual, setManual] = useState(false);
  const [pending, setPending] = useState(false);
  const [appliedPrefill, setAppliedPrefill] = useState(prefill);
  if (prefill !== appliedPrefill) {
    setAppliedPrefill(prefill);
    if (prefill) {
      setTool(prefill.tool ?? "");
      setModel(prefill.model ?? "");
    }
  }

  const selectedTool = detectedTools.includes(tool) ? tool : (detectedTools[0] ?? "");

  const probe = provider ? probes[provider] : undefined;
  const liveModels = probe?.reachable ? probe.models : [];

  const manualAvailable =
    liveModels.length === 0 && (probe?.status === "connected" || probe?.status === "error");
  const valid = Boolean(
    selectedTool &&
    servers.some((server) => server.name === provider) &&
    model.trim() &&
    ((manualAvailable && manual) || liveModels.some((entry) => entry.id === model)),
  );
  useEffect(
    () =>
      onMessage((msg: ExtensionToWebviewMessage) => {
        switch (msg.type) {
          case "delegatorCreated":
            setPending(false);
            setName("");
            setModel("");
            break;
          case "delegatorCreateError":
            setPending(false);
            break;
          case "apiHealthResult":
          case "assessTicketCreated":
          case "assessTicketError":
          case "browseResult":
          case "collectionActivated":
          case "collectionsError":
          case "collectionsLoaded":
          case "configError":
          case "configLoaded":
          case "configUpdated":
          case "externalIssueTypesError":
          case "externalIssueTypesLoaded":
          case "issueTypeCreated":
          case "issueTypeDeleted":
          case "issueTypeError":
          case "issueTypeLoaded":
          case "issueTypeUpdated":
          case "issueTypesLoaded":
          case "jiraValidationResult":
          case "kanbanStatusesError":
          case "kanbanStatusesLoaded":
          case "linearValidationResult":
          case "llmToolsDetected":
          case "modelProvidersError":
          case "modelProvidersLoaded":
          case "navigateTo":
          case "projectsError":
          case "projectsLoaded":
          case "providerProbed":
            break;
        }
      }),
    [],
  );

  const submit = useCallback(() => {
    if (!valid) {
      onInvalid("Pick a tool, a provider, and a model.");
      return;
    }
    if (pending) {
      return;
    }
    setPending(true);
    postMessage({
      type: "createDelegator",
      request: {
        name: name.trim() || `${selectedTool}-${model}`,
        llm_tool: selectedTool,
        model,
        display_name: null,
        model_properties: {},
        model_server: provider,
        launch_config: null,
        remote_agent: null,
        governance: null,
      },
    });
  }, [model, name, onInvalid, pending, provider, selectedTool, valid]);
  const handleToolChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => setTool(event.target.value),
    [setTool],
  );
  const handleProviderChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => {
      setProvider(event.target.value);
      setModel("");
      setManual(false);
    },
    [setModel, setProvider],
  );
  const handleModelChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) =>
      setModel(event.target.value),
    [setModel],
  );
  const handleNameChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => setName(event.target.value),
    [setName],
  );

  return (
    <div className="op-mt-2 op-mb-1">
      <p className="op-body2 op-text-secondary op-mb-1">
        Create delegator - pair a tool with a connected provider and a live model.
      </p>
      <div className="op-col op-delegator-form">
        <SelectInput label="LLM tool" value={selectedTool} onChange={handleToolChange}>
          {detectedTools.length === 0 && <option value="">(none detected)</option>}
          {detectedTools.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </SelectInput>

        <SelectInput label="Provider" value={provider} onChange={handleProviderChange}>
          <option value="">Select...</option>
          {servers.map((k) => (
            <option
              key={k.name}
              value={k.name}
              disabled={!probes[k.name]?.reachable && probes[k.name]?.status !== "error"}
            >
              {k.display_name ?? k.name}
            </option>
          ))}
        </SelectInput>

        {manualAvailable && (
          <label>
            <input
              type="checkbox"
              checked={manual}
              onChange={(event) => setManual(event.target.checked)}
            />{" "}
            Enter model ID manually
          </label>
        )}
        {liveModels.length > 0 ? (
          <SelectInput label="Model" value={model} onChange={handleModelChange}>
            <option value="">Select...</option>
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
            disabled={!manualAvailable || !manual}
            placeholder={provider ? "Model ID" : "Pick a provider first"}
            onChange={handleModelChange}
          />
        )}

        <TextInput
          label="Name (optional)"
          value={name}
          placeholder={selectedTool && model ? `${selectedTool}-${model}` : "delegator name"}
          onChange={handleNameChange}
        />

        <Button
          variant="outlined"
          size="small"
          onClick={submit}
          disabled={pending || !valid}
          className="op-align-start"
        >
          {pending ? "Creating…" : "Create delegator"}
        </Button>
      </div>
    </div>
  );
}
