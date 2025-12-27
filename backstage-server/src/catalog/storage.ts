/**
 * Catalog Entity Storage
 *
 * In-memory storage with optional JSON file persistence.
 * Provides CRUD operations for Backstage catalog entities.
 */

import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';
import { randomUUID } from 'node:crypto';
import type {
  Entity,
  EntityEnvelope,
  Location,
  EntitiesQuery,
  EntitiesResponse,
  EntityFacetsResponse,
} from './types';
import { stringifyEntityRef } from './types';

interface StorageState {
  entities: EntityEnvelope[];
  locations: Location[];
}

export class CatalogStorage {
  private entities: Map<string, EntityEnvelope> = new Map();
  private entitiesByUid: Map<string, EntityEnvelope> = new Map();
  private locations: Map<string, Location> = new Map();
  private persistPath?: string;
  private dirty = false;
  private saveTimeout?: ReturnType<typeof setTimeout>;

  constructor(persistPath?: string) {
    this.persistPath = persistPath;
  }

  async load(): Promise<void> {
    if (!this.persistPath) return;

    try {
      const data = await readFile(this.persistPath, 'utf-8');
      const state: StorageState = JSON.parse(data);

      for (const envelope of state.entities) {
        this.indexEntity(envelope);
      }
      for (const location of state.locations) {
        this.locations.set(location.id, location);
      }

      console.log(`Loaded ${this.entities.size} entities from ${this.persistPath}`);
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code !== 'ENOENT') {
        console.error('Failed to load catalog state:', err);
      }
    }
  }

  private async save(): Promise<void> {
    if (!this.persistPath || !this.dirty) return;

    try {
      await mkdir(dirname(this.persistPath), { recursive: true });

      const state: StorageState = {
        entities: Array.from(this.entities.values()),
        locations: Array.from(this.locations.values()),
      };

      await writeFile(this.persistPath, JSON.stringify(state, null, 2));
      this.dirty = false;
    } catch (err) {
      console.error('Failed to save catalog state:', err);
    }
  }

  private scheduleSave(): void {
    this.dirty = true;
    if (this.saveTimeout) clearTimeout(this.saveTimeout);
    this.saveTimeout = setTimeout(() => this.save(), 1000);
  }

  private indexEntity(envelope: EntityEnvelope): void {
    const ref = stringifyEntityRef(envelope.entity);
    this.entities.set(ref, envelope);

    if (envelope.entity.metadata.uid) {
      this.entitiesByUid.set(envelope.entity.metadata.uid, envelope);
    }
  }

  private unindexEntity(envelope: EntityEnvelope): void {
    const ref = stringifyEntityRef(envelope.entity);
    this.entities.delete(ref);

    if (envelope.entity.metadata.uid) {
      this.entitiesByUid.delete(envelope.entity.metadata.uid);
    }
  }

  // Add or update an entity
  addEntity(entity: Entity, locationKey?: string): Entity {
    // Ensure defaults
    entity.metadata.namespace = entity.metadata.namespace || 'default';
    entity.metadata.uid = entity.metadata.uid || randomUUID();
    entity.metadata.etag = randomUUID().slice(0, 8);

    const envelope: EntityEnvelope = { entity, locationKey };
    this.indexEntity(envelope);
    this.scheduleSave();

    return entity;
  }

  // Remove an entity
  removeEntity(ref: string): boolean {
    const envelope = this.entities.get(ref);
    if (!envelope) return false;

    this.unindexEntity(envelope);
    this.scheduleSave();
    return true;
  }

  // Get entity by reference (kind:namespace/name)
  getEntityByRef(ref: string): Entity | undefined {
    return this.entities.get(ref)?.entity;
  }

  // Get entity by UID
  getEntityByUid(uid: string): Entity | undefined {
    return this.entitiesByUid.get(uid)?.entity;
  }

  // Get entity by kind/namespace/name
  getEntityByName(
    kind: string,
    namespace: string,
    name: string
  ): Entity | undefined {
    const ref = `${kind.toLowerCase()}:${namespace}/${name}`;
    return this.entities.get(ref)?.entity;
  }

  // Query entities with filtering
  queryEntities(query: EntitiesQuery): EntitiesResponse {
    let items = Array.from(this.entities.values()).map((e) => e.entity);

    // Apply filters
    if (query.filter) {
      for (const filter of query.filter) {
        items = this.applyFilter(items, filter);
      }
    }

    // Sort
    if (query.orderField && query.orderField.length > 0) {
      items = this.sortEntities(items, query.orderField);
    }

    const totalItems = items.length;

    // Pagination
    const offset = query.offset || 0;
    const limit = query.limit || 20;
    items = items.slice(offset, offset + limit);

    return {
      items,
      totalItems,
      pageInfo: {
        nextCursor:
          offset + limit < totalItems ? String(offset + limit) : undefined,
        prevCursor: offset > 0 ? String(Math.max(0, offset - limit)) : undefined,
      },
    };
  }

  private applyFilter(entities: Entity[], filter: string): Entity[] {
    // Filter format: field=value or field!=value
    const match = filter.match(/^([^=!]+)(=|!=)(.*)$/);
    if (!match) return entities;

    const [, field, operator, value] = match;

    return entities.filter((entity) => {
      const fieldValue = this.getFieldValue(entity, field);
      const matches =
        Array.isArray(fieldValue)
          ? fieldValue.includes(value)
          : String(fieldValue) === value;

      return operator === '=' ? matches : !matches;
    });
  }

  private getFieldValue(entity: Entity, field: string): unknown {
    const parts = field.split('.');

    // Handle special fields
    if (parts[0] === 'kind') return entity.kind.toLowerCase();
    if (parts[0] === 'metadata') {
      const key = parts.slice(1).join('.');
      return this.getNestedValue(entity.metadata, key);
    }
    if (parts[0] === 'spec') {
      const key = parts.slice(1).join('.');
      return this.getNestedValue(entity.spec || {}, key);
    }

    return undefined;
  }

  private getNestedValue(obj: unknown, path: string): unknown {
    const parts = path.split('.');
    let current: unknown = obj;

    for (const part of parts) {
      if (current == null || typeof current !== 'object') return undefined;
      current = (current as Record<string, unknown>)[part];
    }

    return current;
  }

  private sortEntities(
    entities: Entity[],
    orderFields: Array<{ field: string; order: 'asc' | 'desc' }>
  ): Entity[] {
    return [...entities].sort((a, b) => {
      for (const { field, order } of orderFields) {
        const aVal = String(this.getFieldValue(a, field) ?? '');
        const bVal = String(this.getFieldValue(b, field) ?? '');
        const cmp = aVal.localeCompare(bVal);
        if (cmp !== 0) return order === 'asc' ? cmp : -cmp;
      }
      return 0;
    });
  }

  // Get facet counts
  getFacets(facets: string[]): EntityFacetsResponse {
    const result: Record<string, Array<{ value: string; count: number }>> = {};

    for (const facet of facets) {
      const counts = new Map<string, number>();

      for (const envelope of this.entities.values()) {
        const value = this.getFieldValue(envelope.entity, facet);
        const values = Array.isArray(value) ? value : [value];

        for (const v of values) {
          if (v != null) {
            const key = String(v);
            counts.set(key, (counts.get(key) || 0) + 1);
          }
        }
      }

      result[facet] = Array.from(counts.entries())
        .map(([value, count]) => ({ value, count }))
        .sort((a, b) => b.count - a.count);
    }

    return { facets: result };
  }

  // Location management
  addLocation(type: string, target: string): Location {
    const id = randomUUID();
    const location: Location = { id, type, target };
    this.locations.set(id, location);
    this.scheduleSave();
    return location;
  }

  removeLocation(id: string): boolean {
    const removed = this.locations.delete(id);
    if (removed) this.scheduleSave();
    return removed;
  }

  getLocation(id: string): Location | undefined {
    return this.locations.get(id);
  }

  listLocations(): Location[] {
    return Array.from(this.locations.values());
  }

  // Remove all entities from a location
  removeEntitiesByLocation(locationKey: string): number {
    let removed = 0;
    for (const [ref, envelope] of this.entities) {
      if (envelope.locationKey === locationKey) {
        this.unindexEntity(envelope);
        removed++;
      }
    }
    if (removed > 0) this.scheduleSave();
    return removed;
  }

  // Get all entities (for search indexing)
  getAllEntities(): Entity[] {
    return Array.from(this.entities.values()).map((e) => e.entity);
  }

  // Stats
  getStats(): { entityCount: number; locationCount: number } {
    return {
      entityCount: this.entities.size,
      locationCount: this.locations.size,
    };
  }
}
