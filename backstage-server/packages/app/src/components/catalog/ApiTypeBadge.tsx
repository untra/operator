/**
 * API Type Badge Component
 *
 * Displays a colored badge indicating the API type (OpenAPI, gRPC, GraphQL, etc.)
 */

import React from 'react';
import { Chip, makeStyles } from '@material-ui/core';

// API type color mapping
const API_TYPE_COLORS: Record<string, { background: string; text: string }> = {
  openapi: { background: '#2196F3', text: '#fff' },
  grpc: { background: '#9C27B0', text: '#fff' },
  graphql: { background: '#E91E63', text: '#fff' },
  soap: { background: '#FF9800', text: '#fff' },
  'json-rpc': { background: '#009688', text: '#fff' },
  asyncapi: { background: '#4CAF50', text: '#fff' },
};

const DEFAULT_COLOR = { background: '#607D8B', text: '#fff' };

// Human-readable labels for API types
const API_TYPE_LABELS: Record<string, string> = {
  openapi: 'OpenAPI',
  grpc: 'gRPC',
  graphql: 'GraphQL',
  soap: 'SOAP',
  'json-rpc': 'JSON-RPC',
  asyncapi: 'AsyncAPI',
};

const useStyles = makeStyles({
  badge: {
    height: 20,
    fontSize: '0.7rem',
    fontWeight: 600,
    marginLeft: 8,
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
});

interface ApiTypeBadgeProps {
  apiType?: string;
  className?: string;
}

export function ApiTypeBadge({ apiType, className }: ApiTypeBadgeProps) {
  const classes = useStyles();

  if (!apiType) {
    return null;
  }

  const normalizedType = apiType.toLowerCase();
  const colors = API_TYPE_COLORS[normalizedType] || DEFAULT_COLOR;
  const label = API_TYPE_LABELS[normalizedType] || apiType.toUpperCase();

  return (
    <Chip
      label={label}
      size="small"
      className={`${classes.badge} ${className || ''}`}
      style={{
        backgroundColor: colors.background,
        color: colors.text,
      }}
    />
  );
}

// Check if an entity is an API type
export function isApiEntity(entity: { kind: string; spec?: Record<string, unknown> }): boolean {
  if (entity.kind?.toLowerCase() === 'api') {
    return true;
  }

  // Check Operator taxonomy kinds that map to API
  const specType = entity.spec?.type as string | undefined;
  if (specType) {
    const apiTypes = ['proto-sdk', 'api-gateway', 'api'];
    return apiTypes.includes(specType.toLowerCase());
  }

  return false;
}
