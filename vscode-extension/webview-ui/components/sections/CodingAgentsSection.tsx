import React from 'react';
import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Link from '@mui/material/Link';
import { SectionHeader } from '../SectionHeader';

interface DetectedToolInfo {
  name: string;
  path: string;
  version: string;
  version_ok: boolean;
}

interface CodingAgentsSectionProps {
  agents: Record<string, unknown>;
  llm_tools: Record<string, unknown>;
  onUpdate: (section: string, key: string, value: unknown) => void;
  onDetectTools: () => void;
}

export function CodingAgentsSection({
  agents,
  llm_tools,
  onUpdate,
  onDetectTools,
}: CodingAgentsSectionProps) {
  const maxParallel = Number(agents.max_parallel ?? 2);
  const generationTimeout = Number(agents.generation_timeout_secs ?? 300);
  const stepTimeout = Number(agents.step_timeout ?? 1800);
  const silenceThreshold = Number(agents.silence_threshold ?? 30);
  const rawDetected = llm_tools.detected;
  const detected: DetectedToolInfo[] = Array.isArray(rawDetected)
    ? rawDetected.filter(
        (entry): entry is DetectedToolInfo =>
          typeof entry === 'object' && entry !== null && 'name' in entry
      )
    : [];

  return (
    <Box sx={{ mb: 4 }}>
      <SectionHeader id="section-agents" title="Coding Agents" />
      <Typography color="text.secondary" gutterBottom>
        Configure coding agent behavior and detected LLM tools. For more details see the <Link href="https://operator.untra.io/getting-started/agents/">agents documentation</Link>
      </Typography>

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2.5 }}>
        <Box>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 0.5 }}>
            Detected LLM Tools
          </Typography>
          <Stack direction="row" spacing={1} sx={{ mb: 1 }} flexWrap="wrap" useFlexGap>
            {detected.length > 0 ? (
              detected.map((tool) => (
                <Chip
                  key={tool.name}
                  label={`${tool.name} ${tool.version}`}
                  size="small"
                  color={tool.version_ok ? 'default' : 'warning'}
                  title={tool.path}
                />
              ))
            ) : (
              <Typography variant="body2" color="text.secondary">
                No tools detected
              </Typography>
            )}
          </Stack>
          <Button variant="outlined" size="small" onClick={onDetectTools}>
            Detect Tools
          </Button>
        </Box>

        <TextField
          fullWidth
          size="small"
          type="number"
          label="Max Parallel Agents"
          value={maxParallel}
          onChange={(e) =>
            onUpdate('agents', 'max_parallel', parseInt(e.target.value, 10) || 1)
          }
          inputProps={{ min: 1, max: 16 }}
          helperText="Maximum number of agents running simultaneously"
        />

        <TextField
          fullWidth
          size="small"
          type="number"
          label="Generation Timeout (seconds)"
          value={generationTimeout}
          onChange={(e) =>
            onUpdate(
              'agents',
              'generation_timeout_secs',
              parseInt(e.target.value, 10) || 300
            )
          }
          inputProps={{ min: 30, max: 3600 }}
          helperText="Timeout for each agent generation step"
        />

        <TextField
          fullWidth
          size="small"
          type="number"
          label="Step Timeout (seconds)"
          value={stepTimeout}
          onChange={(e) =>
            onUpdate(
              'agents',
              'step_timeout',
              parseInt(e.target.value, 10) || 1800
            )
          }
          inputProps={{ min: 60, max: 7200 }}
          helperText="Maximum seconds a step can run before timing out"
        />

        <TextField
          fullWidth
          size="small"
          type="number"
          label="Silence Threshold (seconds)"
          value={silenceThreshold}
          onChange={(e) =>
            onUpdate(
              'agents',
              'silence_threshold',
              parseInt(e.target.value, 10) || 30
            )
          }
          inputProps={{ min: 5, max: 300 }}
          helperText="Seconds of silence before considering agent awaiting input"
        />
      </Box>
    </Box>
  );
}
