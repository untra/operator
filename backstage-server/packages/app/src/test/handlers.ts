/**
 * MSW Handlers
 *
 * Default mock handlers for API routes.
 * Each handler returns successful responses by default.
 * Tests can override handlers for error scenarios.
 */

import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

// Default successful responses
export const handlers = [
  // Kanban board endpoint
  http.get('/api/proxy/operator/api/v1/queue/kanban', () => {
    return HttpResponse.json({
      queue: [],
      running: [],
      awaiting: [],
      done: [],
      total_count: 0,
      last_updated: new Date().toISOString(),
    });
  }),

  // Active agents endpoint
  http.get('/api/proxy/operator/api/v1/agents/active', () => {
    return HttpResponse.json({
      agents: [],
      count: 0,
    });
  }),

  // Queue status endpoint
  http.get('/api/proxy/operator/api/v1/queue/status', () => {
    return HttpResponse.json({
      queued: 0,
      in_progress: 0,
      awaiting: 0,
      completed: 0,
      by_type: {
        inv: 0,
        fix: 0,
        feat: 0,
        spike: 0,
      },
    });
  }),

  // Issue types endpoint
  http.get('/api/proxy/operator/api/v1/issuetypes', () => {
    return HttpResponse.json([]);
  }),
];

// Create MSW server instance
export const server = setupServer(...handlers);
