/**
 * Operator Backstage Standalone Server
 *
 * Minimal Bun-based server for local catalog browsing.
 * Uses Hono for HTTP instead of @backstage/backend-defaults
 * to enable bun build --compile.
 */

import { Hono } from 'hono';
import { cors } from 'hono/cors';

const app = new Hono();

// Enable CORS for Operator REST API integration
app.use('/*', cors({
  origin: ['http://localhost:7007', 'http://localhost:7008'],
  allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
}));

// Health check endpoint
app.get('/health', (c) => c.json({ status: 'ok', timestamp: new Date().toISOString() }));

// Status endpoint
app.get('/api/status', (c) => c.json({
  status: 'running',
  version: '1.0.0',
  mode: 'standalone',
}));

// Proxy to Operator REST API (default port 7008)
const operatorApiUrl = process.env.OPERATOR_API_URL || 'http://localhost:7008';

app.all('/api/operator/*', async (c) => {
  const path = c.req.path.replace('/api/operator', '');
  const url = `${operatorApiUrl}${path}`;

  try {
    const response = await fetch(url, {
      method: c.req.method,
      headers: c.req.header(),
      body: c.req.method !== 'GET' ? await c.req.text() : undefined,
    });

    const data = await response.text();
    return new Response(data, {
      status: response.status,
      headers: { 'Content-Type': response.headers.get('Content-Type') || 'application/json' },
    });
  } catch (error) {
    return c.json({ error: 'Operator API unavailable', details: String(error) }, 502);
  }
});

// Issue types endpoint - proxy to operator
app.get('/api/issuetypes', async (c) => {
  try {
    const response = await fetch(`${operatorApiUrl}/api/v1/issuetypes`);
    const data = await response.json();
    return c.json(data);
  } catch (error) {
    return c.json({ error: 'Failed to fetch issue types' }, 500);
  }
});

// Collections endpoint - proxy to operator
app.get('/api/collections', async (c) => {
  try {
    const response = await fetch(`${operatorApiUrl}/api/v1/collections`);
    const data = await response.json();
    return c.json(data);
  } catch (error) {
    return c.json({ error: 'Failed to fetch collections' }, 500);
  }
});

// Fallback - serve a simple status page
app.get('/', (c) => {
  return c.html(`
    <!DOCTYPE html>
    <html>
    <head>
      <title>Operator Backstage</title>
      <style>
        body { font-family: system-ui; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        .status { padding: 20px; background: #f0f0f0; border-radius: 8px; }
        code { background: #e0e0e0; padding: 2px 6px; border-radius: 4px; }
      </style>
    </head>
    <body>
      <h1>Operator Backstage Server</h1>
      <div class="status">
        <p><strong>Status:</strong> Running</p>
        <p><strong>Mode:</strong> Standalone (compiled binary)</p>
        <p><strong>Operator API:</strong> <code>${operatorApiUrl}</code></p>
      </div>
      <h2>Available Endpoints</h2>
      <ul>
        <li><code>GET /health</code> - Health check</li>
        <li><code>GET /api/status</code> - Server status</li>
        <li><code>GET /api/issuetypes</code> - List issue types</li>
        <li><code>GET /api/collections</code> - List collections</li>
        <li><code>ALL /api/operator/*</code> - Proxy to Operator REST API</li>
      </ul>
    </body>
    </html>
  `);
});

// Start server
const port = parseInt(process.env.PORT || '7007');
console.log(`Operator Backstage Server starting on port ${port}...`);
console.log(`Operator API: ${operatorApiUrl}`);

export default {
  port,
  fetch: app.fetch,
};
