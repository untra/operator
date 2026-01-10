/**
 * Search Index
 *
 * Simple in-memory text search index for catalog entities.
 */

import type { Entity } from '../catalog/types';
import { stringifyEntityRef } from '../catalog/types';

interface IndexedDocument {
  entityRef: string;
  kind: string;
  namespace: string;
  name: string;
  title: string;
  description: string;
  tags: string[];
  tier?: string;      // Operator taxonomy tier
  kindType?: string;  // spec.type (taxonomy kind)
  text: string;       // Combined searchable text
}

export interface SearchResult {
  type: string;
  document: {
    title: string;
    text: string;
    location: string;
    kind: string;
    namespace: string;
    name: string;
  };
  rank: number;
}

export interface SearchQuery {
  term: string;
  types?: string[];
  filters?: Record<string, string | string[]>;
}

export class SearchIndex {
  private documents: Map<string, IndexedDocument> = new Map();

  // Index an entity
  indexEntity(entity: Entity): void {
    const ref = stringifyEntityRef(entity);
    const meta = entity.metadata;
    const labels = meta.labels || {};

    // Extract tier and kind type from labels/spec
    const tier = labels['operator-tier'];
    const kindType = entity.spec?.type as string | undefined;

    // Build searchable text from all relevant fields
    const textParts = [
      meta.name,
      meta.title || '',
      meta.description || '',
      ...(meta.tags || []),
      entity.kind,
    ];

    // Add tier to searchable text
    if (tier) {
      textParts.push(tier);
    }

    // Add kindType to searchable text
    if (kindType) {
      textParts.push(kindType);
    }

    // Add spec fields if they're strings
    if (entity.spec) {
      for (const [, value] of Object.entries(entity.spec)) {
        if (typeof value === 'string') {
          textParts.push(value);
        }
      }
    }

    const doc: IndexedDocument = {
      entityRef: ref,
      kind: entity.kind.toLowerCase(),
      namespace: meta.namespace || 'default',
      name: meta.name,
      title: meta.title || meta.name,
      description: meta.description || '',
      tags: meta.tags || [],
      tier,
      kindType,
      text: textParts.join(' ').toLowerCase(),
    };

    this.documents.set(ref, doc);
  }

  // Remove an entity from the index
  removeEntity(entityRef: string): void {
    this.documents.delete(entityRef);
  }

  // Clear and rebuild index
  rebuildIndex(entities: Entity[]): void {
    this.documents.clear();
    for (const entity of entities) {
      this.indexEntity(entity);
    }
  }

  // Search for entities
  search(query: SearchQuery): SearchResult[] {
    const term = query.term.toLowerCase().trim();
    const results: SearchResult[] = [];

    for (const doc of this.documents.values()) {
      // Apply type filter if specified
      if (query.types && query.types.length > 0) {
        const typeMatch = query.types.some(
          (t) => t.toLowerCase() === `software-catalog.${doc.kind}`
        );
        if (!typeMatch) {continue;}
      }

      // Apply kind filter if in filters
      if (query.filters?.kind) {
        const kinds = Array.isArray(query.filters.kind)
          ? query.filters.kind
          : [query.filters.kind];
        if (!kinds.some((k) => k.toLowerCase() === doc.kind)) {continue;}
      }

      // Apply tier filter if in filters
      if (query.filters?.['metadata.labels.operator-tier']) {
        const tiers = Array.isArray(query.filters['metadata.labels.operator-tier'])
          ? query.filters['metadata.labels.operator-tier']
          : [query.filters['metadata.labels.operator-tier']];
        if (!doc.tier || !tiers.includes(doc.tier)) {continue;}
      }

      // Apply kindType (spec.type) filter if in filters
      if (query.filters?.['spec.type']) {
        const kindTypes = Array.isArray(query.filters['spec.type'])
          ? query.filters['spec.type']
          : [query.filters['spec.type']];
        if (!doc.kindType || !kindTypes.includes(doc.kindType)) {continue;}
      }

      // Calculate relevance score
      let rank = 0;

      if (!term) {
        // No search term - return all (with base rank)
        rank = 1;
      } else {
        // Exact name match
        if (doc.name.toLowerCase() === term) {
          rank += 100;
        }
        // Name starts with term
        else if (doc.name.toLowerCase().startsWith(term)) {
          rank += 50;
        }
        // Name contains term
        else if (doc.name.toLowerCase().includes(term)) {
          rank += 25;
        }

        // Title match
        if (doc.title.toLowerCase().includes(term)) {
          rank += 20;
        }

        // Description match
        if (doc.description.toLowerCase().includes(term)) {
          rank += 10;
        }

        // Tags match
        if (doc.tags.some((t) => t.toLowerCase().includes(term))) {
          rank += 15;
        }

        // Tier match
        if (doc.tier?.toLowerCase().includes(term)) {
          rank += 12;
        }

        // Kind type match
        if (doc.kindType?.toLowerCase().includes(term)) {
          rank += 12;
        }

        // General text match
        if (doc.text.includes(term)) {
          rank += 5;
        }
      }

      if (rank > 0) {
        results.push({
          type: `software-catalog.${doc.kind}`,
          document: {
            title: doc.title,
            text: doc.description,
            location: `/catalog/${doc.namespace}/${doc.kind}/${doc.name}`,
            kind: doc.kind,
            namespace: doc.namespace,
            name: doc.name,
          },
          rank,
        });
      }
    }

    // Sort by rank (descending)
    results.sort((a, b) => b.rank - a.rank);

    return results;
  }

  // Get stats
  getStats(): { documentCount: number } {
    return { documentCount: this.documents.size };
  }
}
