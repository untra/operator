import React, { useCallback } from "react";
import { Button, Chip } from "../primitives";
import { SectionHeader } from "../SectionHeader";
import { useDraftField } from "../../hooks/useDraftField";
import type { AgentsConfig } from "../../../src/generated/AgentsConfig";
import type { LlmToolsConfig } from "../../../src/generated/LlmToolsConfig";

const LLM_ICON_NAMES = new Set(["claude", "codex", "gemini"]);

interface NumberFieldProps {
  label: string;
  value: number;
  min: number;
  max: number;
  helperText: string;
  onCommit: (value: number) => void;
}

function parseBoundedInteger(text: string, min: number, max: number): number | null {
  const trimmed = text.trim();
  const n = Number(trimmed);
  if (trimmed === "" || !Number.isInteger(n) || n < min || n > max) {
    return null;
  }
  return n;
}

function NumberField({ label, value, min, max, helperText, onCommit }: NumberFieldProps) {
  const isValid = useCallback(
    (next: string) => parseBoundedInteger(next, min, max) !== null,
    [max, min],
  );
  const commit = useCallback(
    (next: string) => {
      const n = parseBoundedInteger(next, min, max);
      if (n !== null) {
        onCommit(n);
      }
    },
    [max, min, onCommit],
  );
  const draft = useDraftField(String(value), commit, isValid);
  return (
    <label className="op-field">
      <span className="op-field-label">{label}</span>
      <input className="op-field-input" type="number" min={min} max={max} {...draft} />
      <span className="op-field-helper">{helperText}</span>
    </label>
  );
}

interface CodingAgentsSectionProps {
  agents: AgentsConfig;
  llm_tools: LlmToolsConfig;
  onUpdate: (section: string, key: string, value: unknown) => void;
  onDetectTools: () => void;
}

export function CodingAgentsSection({
  agents,
  llm_tools,
  onUpdate,
  onDetectTools,
}: CodingAgentsSectionProps) {
  const maxParallel = agents.max_parallel;
  const generationTimeout = Number(agents.generation_timeout_secs);
  const stepTimeout = Number(agents.step_timeout);
  const silenceThreshold = Number(agents.silence_threshold);
  const detected = llm_tools.detected;
  const commitMaxParallel = useCallback(
    (n: number) => onUpdate("agents", "max_parallel", n),
    [onUpdate],
  );
  const commitGenerationTimeout = useCallback(
    (n: number) => onUpdate("agents", "generation_timeout_secs", BigInt(n)),
    [onUpdate],
  );
  const commitStepTimeout = useCallback(
    (n: number) => onUpdate("agents", "step_timeout", BigInt(n)),
    [onUpdate],
  );
  const commitSilenceThreshold = useCallback(
    (n: number) => onUpdate("agents", "silence_threshold", BigInt(n)),
    [onUpdate],
  );

  return (
    <div className="op-mb-4">
      <SectionHeader id="section-agents" title="Coding Agents" />
      <p className="op-body1 op-text-secondary op-mb-1">
        Configure coding agent behavior and detected LLM tools. For more details see the{" "}
        <a href="https://operator.untra.io/getting-started/agents/">agents documentation</a>
      </p>

      <div className="op-col" style={{ gap: 20 }}>
        <div>
          <p className="op-body2 op-text-secondary op-mb-05">Detected LLM Tools</p>
          <div className="op-row op-gap-1 op-wrap op-mb-1">
            {detected.length > 0 ? (
              detected.map((tool) => (
                <span
                  key={tool.name}
                  title={
                    tool.health_ok ? tool.path : `${tool.path} - health check failed; cannot launch`
                  }
                >
                  <Chip
                    label={
                      <>
                        {LLM_ICON_NAMES.has(tool.name) && (
                          <i
                            className={`opi-${tool.name}`}
                            style={{ fontSize: "1rem", lineHeight: 1 }}
                          />
                        )}
                        {`${tool.name} ${tool.version}`}
                      </>
                    }
                    color={!tool.health_ok ? "error" : tool.version_ok ? "default" : "warning"}
                  />
                </span>
              ))
            ) : (
              <span className="op-body2 op-text-secondary">No tools detected</span>
            )}
          </div>
          <Button variant="outlined" size="small" onClick={onDetectTools}>
            Detect Tools
          </Button>
        </div>

        <NumberField
          label="Max Parallel Agents"
          value={maxParallel}
          min={1}
          max={16}
          onCommit={commitMaxParallel}
          helperText="Maximum number of agents running simultaneously"
        />

        <NumberField
          label="Generation Timeout (seconds)"
          value={generationTimeout}
          min={30}
          max={3600}
          onCommit={commitGenerationTimeout}
          helperText="Timeout for each agent generation step"
        />

        <NumberField
          label="Step Timeout (seconds)"
          value={stepTimeout}
          min={60}
          max={7200}
          onCommit={commitStepTimeout}
          helperText="Maximum seconds a step can run before timing out"
        />

        <NumberField
          label="Silence Threshold (seconds)"
          value={silenceThreshold}
          min={5}
          max={300}
          onCommit={commitSilenceThreshold}
          helperText="Seconds of silence before considering agent awaiting input"
        />
      </div>
    </div>
  );
}
