/**
 * Search REST API Routes
 *
 * Implements the Backstage Search API endpoints.
 */

import { Hono } from 'hono';
import { SearchIndex, type SearchQuery } from './index';

export function createSearchRoutes(searchIndex: SearchIndex): Hono {
  const app = new Hono();

  // POST /query - Search query
  app.post('/query', async (c) => {
    const body = await c.req.json<{
      term: string;
      types?: string[];
      filters?: Record<string, string | string[]>;
      pageLimit?: number;
      pageCursor?: string;
    }>();

    const query: SearchQuery = {
      term: body.term || '',
      types: body.types,
      filters: body.filters,
    };

    let results = searchIndex.search(query);

    // Apply pagination
    const pageLimit = body.pageLimit || 25;
    const offset = body.pageCursor ? parseInt(body.pageCursor, 10) : 0;

    const totalResults = results.length;
    results = results.slice(offset, offset + pageLimit);

    const nextCursor =
      offset + pageLimit < totalResults ? String(offset + pageLimit) : undefined;

    return c.json({
      results,
      nextCursor,
      previousCursor: offset > 0 ? String(Math.max(0, offset - pageLimit)) : undefined,
      numberOfResults: totalResults,
    });
  });

  // GET /query - Alternative GET endpoint
  app.get('/query', async (c) => {
    const url = new URL(c.req.url);
    const term = url.searchParams.get('term') || '';
    const types = url.searchParams.getAll('types');
    const pageLimit = parseInt(url.searchParams.get('pageLimit') || '25', 10);
    const pageCursor = url.searchParams.get('pageCursor') || undefined;

    const query: SearchQuery = { term, types };

    let results = searchIndex.search(query);

    // Apply pagination
    const offset = pageCursor ? parseInt(pageCursor, 10) : 0;
    const totalResults = results.length;
    results = results.slice(offset, offset + pageLimit);

    const nextCursor =
      offset + pageLimit < totalResults ? String(offset + pageLimit) : undefined;

    return c.json({
      results,
      nextCursor,
      previousCursor: offset > 0 ? String(Math.max(0, offset - pageLimit)) : undefined,
      numberOfResults: totalResults,
    });
  });

  // GET / - Search info
  app.get('/', async (c) => {
    const stats = searchIndex.getStats();
    return c.json({
      status: 'ok',
      ...stats,
    });
  });

  return app;
}
