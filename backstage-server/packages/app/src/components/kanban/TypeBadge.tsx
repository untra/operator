/**
 * Type Badge Component
 *
 * Displays the ticket type (FEAT/FIX/INV/SPIKE) with appropriate color.
 */

import React from 'react';
import { Box } from '@backstage/ui';
import { TYPE_BADGE_CONFIG } from './types';

interface TypeBadgeProps {
  type: string;
  size?: 'small' | 'medium';
}

export function TypeBadge({ type, size = 'small' }: TypeBadgeProps) {
  const config = TYPE_BADGE_CONFIG[type] || {
    label: type,
    color: '#888888',
  };

  const padding = size === 'small' ? '2px 6px' : '4px 8px';
  const fontSize = size === 'small' ? '0.625rem' : '0.75rem';

  return (
    <Box
      style={{
        backgroundColor: `${config.color}20`,
        color: config.color,
        padding,
        borderRadius: 4,
        fontSize,
        fontWeight: 600,
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
        display: 'inline-block',
      }}
    >
      {type}
    </Box>
  );
}
