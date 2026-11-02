/**
 * Model-provider picker: one card per provider with its live probe state.
 * Presentational only; the host fetches each probe and passes it in.
 */

import type { ModelServerModelsResponse } from "../generated/ModelServerModelsResponse";
import { Choice, ChoiceGroup } from "./Choice";
import { HealthDot } from "./HealthDot";
import styles from "./ModelProviderChoices.module.css";

export interface ProbeState {
  /** Probe response; absent while the request is in flight. */
  probe?: ModelServerModelsResponse;
  /** The request to Operator itself failed. */
  fetchFailed?: boolean;
  /** Env var the provider reads its key from. */
  keyEnv?: string;
  /** Server runs beside Operator, so unreachable means not started. */
  local?: boolean;
}

export interface ProbeLabel {
  health: "green" | "yellow" | "red" | "gray";
  text: string;
}

export function probeStatusLabel({ probe, fetchFailed, keyEnv, local }: ProbeState): ProbeLabel {
  if (fetchFailed) {
    return { health: "red", text: "probe failed" };
  }
  switch (probe?.status) {
    case undefined:
      return { health: "gray", text: "checking…" };
    case "connected":
      return { health: "green", text: `${probe.models.length} models` };
    case "missing_key":
      return { health: "gray", text: keyEnv ? `set ${keyEnv}` : "API key not set" };
    case "unauthorized":
      return { health: "red", text: "API key rejected" };
    case "unreachable":
      return { health: "yellow", text: local ? "not running" : "unreachable" };
    default:
      return { health: "red", text: "probe failed" };
  }
}

export interface ModelProviderChoice extends ProbeState {
  slug: string;
  label: string;
}

export interface ModelProviderCardProps {
  provider: ModelProviderChoice;
  selected: boolean;
  onToggle: (slug: string) => void;
}

/** One provider card; place inside a `ChoiceGroup`. */
export function ModelProviderCard({ provider, selected, onToggle }: ModelProviderCardProps) {
  const label = probeStatusLabel(provider);
  return (
    <Choice selected={selected} value={provider.slug} onSelect={onToggle}>
      <strong>{provider.label}</strong>
      <span className={styles.status} title={provider.probe?.error ?? undefined}>
        <HealthDot health={label.health} />
        {label.text}
      </span>
    </Choice>
  );
}

export interface ModelProviderChoicesProps {
  providers: ModelProviderChoice[];
  selected: string[];
  onToggle: (slug: string) => void;
}

export function ModelProviderChoices({ providers, selected, onToggle }: ModelProviderChoicesProps) {
  return (
    <ChoiceGroup>
      {providers.map((provider) => (
        <ModelProviderCard
          key={provider.slug}
          provider={provider}
          selected={selected.includes(provider.slug)}
          onToggle={onToggle}
        />
      ))}
    </ChoiceGroup>
  );
}
