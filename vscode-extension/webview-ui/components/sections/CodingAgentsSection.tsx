import React from 'react';
import { Button, Chip } from '../primitives';
import { SectionHeader } from '../SectionHeader';
import type { AgentsConfig } from '../../../src/generated/AgentsConfig';
import type { LlmToolsConfig } from '../../../src/generated/LlmToolsConfig';

const LLM_ICON_NAMES = ['claude', 'codex', 'gemini'];

interface NumberFieldProps {
  label: string;
  value: number;
  min: number;
  max: number;
  helperText: string;
  onChange: React.ChangeEventHandler<HTMLInputElement>;
}

function NumberField({ label, value, min, max, helperText, onChange }: NumberFieldProps) {
  return (
    <label className="op-field">
      <span className="op-field-label">{label}</span>
      <input
        className="op-field-input"
        type="number"
        value={value}
        min={min}
        max={max}
        onChange={onChange}
      />
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

  return (
    <div className="op-mb-4">
      <SectionHeader id="section-agents" title="Coding Agents" />
      <p className="op-body1 op-text-secondary op-mb-1">
        Configure coding agent behavior and detected LLM tools. For more details see the <a href="https://operator.untra.io/getting-started/agents/">agents documentation</a>
      </p>

      <div className="op-col" style={{ gap: 20 }}>
        <div>
          <p className="op-body2 op-text-secondary op-mb-05">
            Detected LLM Tools
          </p>
          <div className="op-row op-gap-1 op-wrap op-mb-1">
            {detected.length > 0 ? (
              detected.map((tool) => (
                <span
                  key={tool.name}
                  title={tool.health_ok ? tool.path : `${tool.path} — health check failed; cannot launch`}
                >
                  <Chip
                    label={
                      <>
                        {LLM_ICON_NAMES.includes(tool.name) && (
                          <i className={`opi-${tool.name}`} style={{ fontSize: '1rem', lineHeight: 1 }} />
                        )}
                        {`${tool.name} ${tool.version}`}
                      </>
                    }
                    color={!tool.health_ok ? 'error' : tool.version_ok ? 'default' : 'warning'}
                  />
                </span>
              ))
            ) : (
              <span className="op-body2 op-text-secondary">
                No tools detected
              </span>
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
          onChange={(e) =>
            onUpdate('agents', 'max_parallel', parseInt(e.target.value, 10) || 1)
          }
          helperText="Maximum number of agents running simultaneously"
        />

        <NumberField
          label="Generation Timeout (seconds)"
          value={generationTimeout}
          min={30}
          max={3600}
          onChange={(e) =>
            onUpdate(
              'agents',
              'generation_timeout_secs',
              parseInt(e.target.value, 10) || 300
            )
          }
          helperText="Timeout for each agent generation step"
        />

        <NumberField
          label="Step Timeout (seconds)"
          value={stepTimeout}
          min={60}
          max={7200}
          onChange={(e) =>
            onUpdate(
              'agents',
              'step_timeout',
              parseInt(e.target.value, 10) || 1800
            )
          }
          helperText="Maximum seconds a step can run before timing out"
        />

        <NumberField
          label="Silence Threshold (seconds)"
          value={silenceThreshold}
          min={5}
          max={300}
          onChange={(e) =>
            onUpdate(
              'agents',
              'silence_threshold',
              parseInt(e.target.value, 10) || 30
            )
          }
          helperText="Seconds of silence before considering agent awaiting input"
        />
      </div>
    </div>
  );
}
