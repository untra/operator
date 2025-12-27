/**
 * Backstage Catalog Entity Types
 *
 * Minimal type definitions matching the Backstage catalog API.
 * @see https://backstage.io/docs/features/software-catalog/descriptor-format
 */

export interface EntityMeta {
  uid?: string;
  etag?: string;
  name: string;
  namespace?: string;
  title?: string;
  description?: string;
  labels?: Record<string, string>;
  annotations?: Record<string, string>;
  tags?: string[];
  links?: Array<{
    url: string;
    title?: string;
    icon?: string;
    type?: string;
  }>;
}

export interface EntityRelation {
  type: string;
  targetRef: string;
}

export interface Entity {
  apiVersion: string;
  kind: string;
  metadata: EntityMeta;
  spec?: Record<string, unknown>;
  relations?: EntityRelation[];
}

export interface EntityEnvelope {
  entity: Entity;
  locationKey?: string;
}

export interface Location {
  id: string;
  type: string;
  target: string;
}

// Standard Backstage entity kinds
export type EntityKind =
  | 'Component'
  | 'API'
  | 'Resource'
  | 'System'
  | 'Domain'
  | 'Group'
  | 'User'
  | 'Location'
  | 'Template';

// Entity reference format: [kind:]namespace/name
export function parseEntityRef(ref: string): {
  kind?: string;
  namespace: string;
  name: string;
} {
  const parts = ref.split('/');
  if (parts.length === 1) {
    return { namespace: 'default', name: parts[0] };
  }
  if (parts.length === 2) {
    const [kindOrNs, name] = parts;
    if (kindOrNs.includes(':')) {
      const [kind, namespace] = kindOrNs.split(':');
      return { kind, namespace, name };
    }
    return { namespace: kindOrNs, name };
  }
  return { namespace: 'default', name: ref };
}

export function stringifyEntityRef(entity: Entity): string {
  const namespace = entity.metadata.namespace || 'default';
  return `${entity.kind.toLowerCase()}:${namespace}/${entity.metadata.name}`;
}

// Query parameters for catalog API
export interface EntitiesQuery {
  filter?: string[];
  fields?: string[];
  offset?: number;
  limit?: number;
  after?: string;
  orderField?: Array<{ field: string; order: 'asc' | 'desc' }>;
}

// API response types
export interface EntitiesResponse {
  items: Entity[];
  totalItems: number;
  pageInfo: {
    nextCursor?: string;
    prevCursor?: string;
  };
}

export interface EntityFacetsResponse {
  facets: Record<string, Array<{ value: string; count: number }>>;
}
