import React from 'react';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Tooltip from '@mui/material/Tooltip';
import type { StepResponse } from '../../../src/generated/StepResponse';

interface WorkflowPreviewProps {
  steps: StepResponse[];
  compact?: boolean;
}

const MODE_COLORS: Record<string, string> = {
  acceptEdits: '#4caf50',
  default: '#4caf50',
  plan: '#2196f3',
  delegate: '#ff9800',
};

function buildStepChain(steps: StepResponse[]): StepResponse[] {
  if (steps.length === 0) {
    return [];
  }

  // Find step with no incoming next_step references (first step)
  const referencedNames = new Set(steps.map(s => s.next_step).filter(Boolean));
  let first = steps.find(s => !referencedNames.has(s.name));
  if (!first) {
    first = steps[0];
  }

  const chain: StepResponse[] = [];
  const visited = new Set<string>();
  let current: StepResponse | undefined = first;

  while (current && !visited.has(current.name)) {
    chain.push(current);
    visited.add(current.name);
    current = current.next_step
      ? steps.find(s => s.name === current!.next_step)
      : undefined;
  }

  // Add any remaining steps not in chain
  for (const s of steps) {
    if (!visited.has(s.name)) {
      chain.push(s);
    }
  }

  return chain;
}

export function WorkflowPreview({ steps, compact = false }: WorkflowPreviewProps) {
  const chain = buildStepChain(steps);

  if (chain.length === 0) {
    return (
      <Typography variant="caption" color="text.secondary">
        No workflow steps defined
      </Typography>
    );
  }

  if (compact) {
    return (
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5, flexWrap: 'wrap' }}>
        {chain.map((step, i) => (
          <React.Fragment key={step.name}>
            {i > 0 && (
              <Typography variant="caption" color="text.secondary" sx={{ mx: 0.25 }}>
                →
              </Typography>
            )}
            <Tooltip
              title={
                <Box>
                  <Typography variant="caption" display="block">
                    {step.display_name || step.name}
                  </Typography>
                  <Typography variant="caption" display="block" color="text.secondary">
                    Mode: {step.permission_mode} | Review: {step.review_type}
                  </Typography>
                  {step.outputs.length > 0 && (
                    <Typography variant="caption" display="block" color="text.secondary">
                      Outputs: {step.outputs.join(', ')}
                    </Typography>
                  )}
                  {step.prompt && (
                    <Typography variant="caption" display="block" color="text.secondary" sx={{ mt: 0.5 }}>
                      {step.prompt.substring(0, 100)}{step.prompt.length > 100 ? '...' : ''}
                    </Typography>
                  )}
                </Box>
              }
              arrow
            >
              <Box
                sx={{
                  px: 0.75,
                  py: 0.25,
                  borderRadius: 1,
                  bgcolor: MODE_COLORS[step.permission_mode] || MODE_COLORS.default,
                  color: '#fff',
                  fontSize: '0.7rem',
                  fontWeight: 500,
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: 0.5,
                  cursor: 'default',
                }}
              >
                {step.display_name || step.name}
                {step.review_type !== 'none' && ' ★'}
              </Box>
            </Tooltip>
          </React.Fragment>
        ))}
      </Box>
    );
  }

  // Vertical step list
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 0 }}>
      {chain.map((step, i) => (
        <Box
          key={step.name}
          sx={{
            borderLeft: `3px solid ${MODE_COLORS[step.permission_mode] || MODE_COLORS.default}`,
            pl: 1.5,
            py: 0.75,
            borderTop: i === 0 ? '1px solid' : undefined,
            borderBottom: '1px solid',
            borderColor: 'divider',
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Typography variant="body2" fontWeight={600}>
              {i + 1}. {step.display_name || step.name}
            </Typography>
            {step.review_type !== 'none' && (
              <Typography variant="caption" color="warning.main">★ review</Typography>
            )}
          </Box>
          <Box sx={{ display: 'flex', gap: 2, mt: 0.25 }}>
            <Typography variant="caption" color="text.secondary">
              Mode: {step.permission_mode}
            </Typography>
            {step.outputs.length > 0 && (
              <Typography variant="caption" color="text.secondary">
                Outputs: {step.outputs.join(', ')}
              </Typography>
            )}
          </Box>
        </Box>
      ))}
    </Box>
  );
}
