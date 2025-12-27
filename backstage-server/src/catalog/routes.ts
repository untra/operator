/**
 * Catalog REST API Routes
 *
 * Implements the Backstage Catalog API endpoints for the React frontend.
 * @see https://backstage.io/docs/features/software-catalog/software-catalog-api
 */

import { Hono } from 'hono';
import { CatalogStorage } from './storage';
import type { EntitiesQuery, Entity } from './types';
import { parseEntityRef } from './types';

export function createCatalogRoutes(storage: CatalogStorage): Hono {
  const app = new Hono();

  // GET /entities/by-query - Query entities with filtering
  app.get('/entities/by-query', async (c) => {
    const url = new URL(c.req.url);

    // Collect filters from both formats:
    // - Standard: filter=field=value
    // - Backstage: filters[field]=value
    const filters: string[] = url.searchParams.getAll('filter');

    // Parse Backstage-style filters[field]=value
    for (const [key, value] of url.searchParams.entries()) {
      const match = key.match(/^filters\[(.+)\]$/);
      if (match) {
        const field = match[1];
        filters.push(`${field}=${value}`);
      }
    }

    const query: EntitiesQuery = {
      filter: filters,
      fields: url.searchParams.getAll('fields'),
      offset: parseInt(url.searchParams.get('offset') || '0', 10),
      limit: parseInt(url.searchParams.get('limit') || '20', 10),
    };

    // Parse orderField if present
    const orderFields = url.searchParams.getAll('orderField');
    if (orderFields.length > 0) {
      query.orderField = orderFields.map((f) => {
        const [field, order] = f.split(',');
        return { field, order: (order as 'asc' | 'desc') || 'asc' };
      });
    }

    const result = storage.queryEntities(query);
    return c.json(result);
  });

  // GET /entities - List all entities (legacy endpoint)
  app.get('/entities', async (c) => {
    const url = new URL(c.req.url);

    // Collect filters from both formats
    const filters: string[] = url.searchParams.getAll('filter');
    for (const [key, value] of url.searchParams.entries()) {
      const match = key.match(/^filters\[(.+)\]$/);
      if (match) {
        filters.push(`${match[1]}=${value}`);
      }
    }

    const query: EntitiesQuery = {
      filter: filters,
      offset: parseInt(url.searchParams.get('offset') || '0', 10),
      limit: parseInt(url.searchParams.get('limit') || '500', 10),
    };

    const result = storage.queryEntities(query);
    return c.json(result.items);
  });

  // GET /entities/by-uid/:uid - Get entity by UID
  app.get('/entities/by-uid/:uid', async (c) => {
    const uid = c.req.param('uid');
    const entity = storage.getEntityByUid(uid);

    if (!entity) {
      return c.json({ error: 'Entity not found' }, 404);
    }

    return c.json(entity);
  });

  // GET /entities/by-name/:kind/:namespace/:name - Get entity by name
  app.get('/entities/by-name/:kind/:namespace/:name', async (c) => {
    const { kind, namespace, name } = c.req.param();
    const entity = storage.getEntityByName(kind, namespace, name);

    if (!entity) {
      return c.json({ error: 'Entity not found' }, 404);
    }

    return c.json(entity);
  });

  // POST /entities/by-refs - Batch get entities by refs
  app.post('/entities/by-refs', async (c) => {
    const body = await c.req.json<{ entityRefs: string[]; fields?: string[] }>();
    const { entityRefs } = body;

    const items = entityRefs.map((ref) => {
      const parsed = parseEntityRef(ref);
      if (parsed.kind) {
        return storage.getEntityByName(
          parsed.kind,
          parsed.namespace,
          parsed.name
        );
      }
      return storage.getEntityByRef(ref);
    });

    return c.json({ items: items.filter(Boolean) });
  });

  // GET /entity-facets - Get facet counts
  app.get('/entity-facets', async (c) => {
    const url = new URL(c.req.url);
    const facets = url.searchParams.getAll('facet');

    if (facets.length === 0) {
      return c.json({ facets: {} });
    }

    const result = storage.getFacets(facets);
    return c.json(result);
  });

  // GET /locations - List all locations
  app.get('/locations', async (c) => {
    const locations = storage.listLocations();
    return c.json(locations);
  });

  // POST /locations - Register a new location
  app.post('/locations', async (c) => {
    const body = await c.req.json<{ type: string; target: string }>();
    const { type, target } = body;

    if (!type || !target) {
      return c.json({ error: 'type and target are required' }, 400);
    }

    const location = storage.addLocation(type, target);
    return c.json(location, 201);
  });

  // DELETE /locations/:id - Remove a location
  app.delete('/locations/:id', async (c) => {
    const id = c.req.param('id');
    const removed = storage.removeLocation(id);

    if (!removed) {
      return c.json({ error: 'Location not found' }, 404);
    }

    return c.json({ success: true });
  });

  // POST /refresh - Refresh an entity (trigger re-ingestion)
  app.post('/refresh', async (c) => {
    const body = await c.req.json<{ entityRef: string }>();
    const { entityRef } = body;

    // For now, just acknowledge the refresh request
    // In a full implementation, this would trigger re-ingestion
    console.log(`Refresh requested for: ${entityRef}`);

    return c.json({ success: true });
  });

  // POST /entities - Create/update entity directly
  app.post('/entities', async (c) => {
    const entity = await c.req.json<Entity>();

    if (!entity.apiVersion || !entity.kind || !entity.metadata?.name) {
      return c.json(
        { error: 'apiVersion, kind, and metadata.name are required' },
        400
      );
    }

    const saved = storage.addEntity(entity);
    return c.json(saved, 201);
  });

  // DELETE /entities/by-uid/:uid - Delete entity by UID
  app.delete('/entities/by-uid/:uid', async (c) => {
    const uid = c.req.param('uid');
    const entity = storage.getEntityByUid(uid);

    if (!entity) {
      return c.json({ error: 'Entity not found' }, 404);
    }

    const ref = `${entity.kind.toLowerCase()}:${entity.metadata.namespace || 'default'}/${entity.metadata.name}`;
    storage.removeEntity(ref);

    return c.json({ success: true });
  });

  // GET / - Catalog info
  app.get('/', async (c) => {
    const stats = storage.getStats();
    return c.json({
      status: 'ok',
      ...stats,
    });
  });

  return app;
}
