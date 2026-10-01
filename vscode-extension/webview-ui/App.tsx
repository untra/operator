import React, { useEffect, useState, useCallback, useRef } from "react";
import { Alert, Spinner } from "./components/primitives";
import { ConfigPage } from "./components/ConfigPage";
import { postMessage, onMessage } from "./vscodeApi";
import { DEFAULT_WEBVIEW_CONFIG } from "./types/defaults";
import type {
  WebviewConfig,
  ExtensionToWebviewMessage,
  JiraValidationInfo,
  LinearValidationInfo,
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
  NavigationPrefill,
} from "./types/messages";
import { applyUpdate } from "./state/applyUpdate";
import {
  configErrorFrom,
  errorAfterSnapshot,
  isStaleSnapshot,
  toWireValue,
  type ConfigErrorState,
} from "./state/configSync";

interface NavigationRequest {
  section: string;
  prefill?: NavigationPrefill;
}

const CREATE_DELEGATOR_ACTION = "createDelegator";

function browseFolder(field: string): void {
  postMessage({ type: "browseFolder", field });
}

function openFile(filePath: string): void {
  postMessage({ type: "openFile", filePath });
}

function startSetup(): void {
  postMessage({ type: "openWalkthrough" });
}

function detectTools(): void {
  postMessage({ type: "detectLlmTools" });
}

function getExternalIssueTypes(provider: string, domain: string, projectKey: string): void {
  postMessage({ type: "getExternalIssueTypes", provider, domain, projectKey });
}

function getKanbanStatuses(provider: string, projectKey: string): void {
  postMessage({ type: "getKanbanStatuses", provider, projectKey });
}

function openOperatorUi(route: "issuetypes" | "projects"): void {
  postMessage({ type: "openOperatorUi", route });
}

export function App() {
  const [config, setConfig] = useState<WebviewConfig | null>(null);
  const [error, setError] = useState<ConfigErrorState | null>(null);
  const [jiraResult, setJiraResult] = useState<JiraValidationInfo | null>(null);
  const [linearResult, setLinearResult] = useState<LinearValidationInfo | null>(null);
  const [validatingJira, setValidatingJira] = useState(false);
  const [validatingLinear, setValidatingLinear] = useState(false);
  const [apiReachable, setApiReachable] = useState(false);
  const [issueTypes, setIssueTypes] = useState<IssueTypeSummary[]>([]);
  const [collections, setCollections] = useState<CollectionResponse[]>([]);
  const [externalIssueTypes, setExternalIssueTypes] = useState<
    Map<string, ExternalIssueTypeSummary[]>
  >(new Map());
  const [kanbanStatuses, setKanbanStatuses] = useState<Map<string, string[]>>(new Map());
  const [navigation, setNavigation] = useState<NavigationRequest | null>(null);
  const lastSentRev = useRef(0);
  const hasConfig = config !== null;
  const delegatorPrefill =
    navigation?.prefill?.action === CREATE_DELEGATOR_ACTION ? navigation.prefill : undefined;

  useEffect(() => {
    const cleanup = onMessage((msg: ExtensionToWebviewMessage) => {
      switch (msg.type) {
        case "configLoaded":
        case "configUpdated":
          setError((prev) => errorAfterSnapshot(prev, msg.type));
          if (!isStaleSnapshot(msg.rev, lastSentRev.current)) {
            setConfig(mergeWithDefaults(msg.config));
          }
          break;
        case "configError":
          setError(configErrorFrom(msg.error, msg.rev));
          if (msg.rev !== undefined) {
            postMessage({ type: "getConfig" });
          }
          break;
        case "browseResult":
          setConfig((prev) => {
            if (!prev) {
              return prev;
            }
            if (msg.field === "workingDirectory") {
              return { ...prev, working_directory: msg.path };
            }
            return prev;
          });
          break;
        case "jiraValidationResult":
          setJiraResult(msg.result);
          setValidatingJira(false);
          break;
        case "linearValidationResult":
          setLinearResult(msg.result);
          setValidatingLinear(false);
          break;
        case "llmToolsDetected":
          if (!isStaleSnapshot(msg.rev, lastSentRev.current)) {
            setConfig(mergeWithDefaults(msg.config));
          }
          break;
        case "navigateTo":
          setNavigation({ section: msg.section, prefill: msg.prefill });
          break;
        case "apiHealthResult":
          setApiReachable(msg.reachable);
          if (msg.reachable) {
            postMessage({ type: "getIssueTypes" });
            postMessage({ type: "getCollections" });
          }
          break;
        case "issueTypesLoaded":
          setIssueTypes(msg.issueTypes);
          break;
        case "collectionsLoaded":
          setCollections(msg.collections);
          break;
        case "externalIssueTypesLoaded":
          setExternalIssueTypes((prev) => {
            const next = new Map(prev);
            next.set(`${msg.provider}/${msg.projectKey}`, msg.types);
            return next;
          });
          break;
        case "externalIssueTypesError":
          // External issue type lookup failed; the mapping panel renders an
          // empty/unmapped state, so no extra handling is required here.
          break;
        case "kanbanStatusesLoaded":
          setKanbanStatuses((prev) => {
            const next = new Map(prev);
            next.set(`${msg.provider}/${msg.projectKey}`, msg.statuses);
            return next;
          });
          break;
        case "kanbanStatusesError":
          // Status discovery failed; the dropdowns fall back to free-form
          // entry of the current mapping values, so no extra handling here.
          break;
        case "assessTicketCreated":
        case "assessTicketError":
        case "collectionActivated":
        case "collectionsError":
        case "delegatorCreated":
        case "delegatorCreateError":
        case "issueTypeCreated":
        case "issueTypeDeleted":
        case "issueTypeError":
        case "issueTypeLoaded":
        case "issueTypeUpdated":
        case "modelProvidersError":
        case "modelProvidersLoaded":
        case "projectsError":
        case "projectsLoaded":
        case "providerProbed":
          break;
      }
    });

    // Signal ready and request config
    postMessage({ type: "ready" });
    postMessage({ type: "getConfig" });
    postMessage({ type: "checkApiHealth" });

    return cleanup;
  }, []);

  useEffect(() => {
    if (hasConfig && navigation) {
      document
        .getElementById(navigation.section)
        ?.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  }, [hasConfig, navigation]);

  const handleUpdate = useCallback(
    (section: string, key: string, value: unknown, instanceKey?: string) => {
      lastSentRev.current += 1;
      const rev = lastSentRev.current;
      postMessage({
        type: "updateConfig",
        section,
        key,
        value: toWireValue(value),
        rev,
        instanceKey,
      });
      setConfig((prev) => (prev ? applyUpdate(prev, section, key, value, instanceKey) : prev));
    },
    [setConfig],
  );

  const handleValidateJira = useCallback(
    (domain: string, email: string, apiToken: string) => {
      setValidatingJira(true);
      setJiraResult(null);
      postMessage({ type: "validateJira", domain, email, apiToken });
    },
    [setJiraResult, setValidatingJira],
  );

  const handleValidateLinear = useCallback(
    (apiKey: string) => {
      setValidatingLinear(true);
      setLinearResult(null);
      postMessage({ type: "validateLinear", apiKey });
    },
    [setLinearResult, setValidatingLinear],
  );

  return (
    <>
      {error && (
        <Alert severity="error" style={{ margin: 16 }}>
          {error.message}
        </Alert>
      )}
      {config ? (
        <ConfigPage
          config={config}
          onUpdate={handleUpdate}
          onBrowseFolder={browseFolder}
          onOpenFile={openFile}
          onStartSetup={startSetup}
          onValidateJira={handleValidateJira}
          onValidateLinear={handleValidateLinear}
          onDetectTools={detectTools}
          jiraResult={jiraResult}
          linearResult={linearResult}
          validatingJira={validatingJira}
          validatingLinear={validatingLinear}
          apiReachable={apiReachable}
          issueTypes={issueTypes}
          collections={collections}
          externalIssueTypes={externalIssueTypes}
          onGetExternalIssueTypes={getExternalIssueTypes}
          kanbanStatuses={kanbanStatuses}
          onGetKanbanStatuses={getKanbanStatuses}
          onOpenOperatorUi={openOperatorUi}
          delegatorPrefill={delegatorPrefill}
        />
      ) : (
        <div
          className="op-col op-gap-2"
          style={{ alignItems: "center", justifyContent: "center", height: "100vh" }}
        >
          <Spinner />
          <p className="op-body2 op-text-secondary">Loading configuration...</p>
        </div>
      )}
    </>
  );
}

/** Deep-merge incoming config with defaults so all fields exist */
function mergeWithDefaults(incoming: WebviewConfig): WebviewConfig {
  const defaults = DEFAULT_WEBVIEW_CONFIG;
  return {
    config_path: incoming.config_path || defaults.config_path,
    working_directory: incoming.working_directory || defaults.working_directory,
    config_exists: incoming.config_exists ?? defaults.config_exists,
    config: deepMerge(defaults.config, incoming.config),
  };
}

/** Recursively merge source into target (source wins for leaf values) */
function deepMerge<T extends Record<string, unknown>>(target: T, source: T): T {
  const result: Record<string, unknown> = { ...target };
  for (const key of Object.keys(source)) {
    const srcVal = (source as Record<string, unknown>)[key];
    const tgtVal = (target as Record<string, unknown>)[key];
    if (
      srcVal !== null &&
      srcVal !== undefined &&
      typeof srcVal === "object" &&
      !Array.isArray(srcVal) &&
      typeof tgtVal === "object" &&
      tgtVal !== null &&
      !Array.isArray(tgtVal)
    ) {
      result[key] = deepMerge(tgtVal as Record<string, unknown>, srcVal as Record<string, unknown>);
    } else if (srcVal !== undefined) {
      result[key] = srcVal;
    }
  }
  return result as T;
}
