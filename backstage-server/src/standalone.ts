/**
 * Operator Backstage Standalone Server
 *
 * Bun-based server for the Backstage catalog and developer portal.
 * Uses Hono for HTTP instead of @backstage/backend-defaults
 * to enable bun build --compile.
 *
 * Features:
 * - In-memory catalog with file persistence
 * - Search index for catalog entities
 * - Proxy to Operator REST API
 * - Embedded React frontend
 */

// Import embedded assets first - enables Bun to embed frontend files into the binary
import './embedded-assets';

import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { join } from 'node:path';
import { homedir } from 'node:os';
import { readFile } from 'node:fs/promises';

import { CatalogStorage, createCatalogRoutes } from './catalog';
import { SearchIndex } from './search/index';
import { createSearchRoutes } from './search/routes';

// Branding/theme configuration interface
interface ThemeConfig {
  appTitle: string;
  orgName: string;
  logoPath?: string;
  mode: 'light' | 'dark' | 'system';
  colors: {
    // Core brand colors
    primary: string;      // Main action color (Terracotta)
    secondary: string;    // Secondary elements (Deep Pine)
    accent: string;       // Highlights, light surfaces (Cream)
    warning: string;      // Alerts
    muted: string;        // Subdued text (Cornflower)
    // Light mode surfaces
    background: string;   // Page background
    surface: string;      // Card/paper background
    text: string;         // Primary text color
    // Navigation scale (4 levels, L1=lightest, L4=darkest)
    navL1: string;        // Nav button default (Sage)
    navL2: string;        // Nav hover (Teal)
    navL3: string;        // Nav selected (Deep Pine)
    navL4: string;        // Nav background/darkest (Midnight)
  };
  components?: {
    borderRadius?: number;  // Default: 4
  };
}

// Build asset map from embedded files
const assetMap = new Map<string, Blob>();

// Bun embeds files with a `name` property containing the path
interface BunBlob extends Blob {
  name: string;
}

const embeddedFiles = (
  (globalThis as unknown as { Bun?: { embeddedFiles?: readonly BunBlob[] } }).Bun?.embeddedFiles ?? []
) as BunBlob[];

for (const blob of embeddedFiles) {
  // Bun embeds with paths like "./assets/static/main.js" or "assets/index.html"
  // Normalize to URL path by removing leading "./" and "assets/" prefix
  const name = blob.name
    .replace(/^\.\//, '')           // Remove leading ./
    .replace(/^assets\//, '');       // Remove assets/ prefix
  assetMap.set(name, blob);
}

const hasEmbeddedFrontend = assetMap.has('index.html');

// Initialize catalog storage with persistence
const catalogPersistPath = join(homedir(), '.operator', 'backstage-catalog.json');
const catalogStorage = new CatalogStorage(catalogPersistPath);
await catalogStorage.load();

// Initialize search index
const searchIndex = new SearchIndex();

// Index existing entities
for (const entity of catalogStorage.getAllEntities()) {
  searchIndex.indexEntity(entity);
}

// Load branding configuration from ~/.operator/backstage/branding/theme.json
const brandingPath = join(homedir(), '.operator', 'backstage', 'branding');
const themePath = join(brandingPath, 'theme.json');

// Default theme config (matches docs/assets/css/main.css)
let themeConfig: ThemeConfig = {
  appTitle: 'Operator!',
  orgName: 'Operator',
  logoPath: 'logo.svg',
  mode: 'system',  // Respects OS light/dark preference
  colors: {
    // Core brand (from docs palette)
    primary: '#E05D44',     // Terracotta
    secondary: '#115566',   // Deep Pine
    accent: '#F2EAC9',      // Cream
    warning: '#E05D44',     // Terracotta
    muted: '#6688AA',       // Cornflower
    // Light mode surfaces
    background: '#faf8f5',  // Warm off-white
    surface: '#ffffff',     // Pure white cards
    text: '#115566',        // Deep Pine
    // Navigation green scale (L1=lightest, L4=darkest)
    navL1: '#66AA99',       // Sage - button default
    navL2: '#448880',       // Teal - hover
    navL3: '#115566',       // Deep Pine - selected
    navL4: '#082226',       // Midnight - nav background
  },
  components: {
    borderRadius: 4,        // Subtle rounding
  },
};

// Try to load custom theme config (deep merge for partial configs)
try {
  const data = await readFile(themePath, 'utf-8');
  const loaded = JSON.parse(data);
  themeConfig = {
    ...themeConfig,
    ...loaded,
    colors: {
      ...themeConfig.colors,
      ...(loaded.colors || {}),
    },
    components: {
      ...themeConfig.components,
      ...(loaded.components || {}),
    },
  };
  console.log(`Loaded branding config from ${themePath}`);
} catch {
  console.log('Using default branding config (no theme.json found)');
}

// Add sample entities if catalog is empty (for demo purposes)
// These demonstrate the 5-tier taxonomy model
if (catalogStorage.getStats().entityCount === 0) {
  const sampleEntities = [
    // Ecosystem tier - CLI/Developer Tools (ID 21)
    {
      apiVersion: 'backstage.io/v1alpha1',
      kind: 'Component',
      metadata: {
        name: 'operator',
        namespace: 'default',
        title: 'Operator TUI',
        description: 'Rust TUI application for orchestrating Claude Code agents',
        labels: {
          'operator-tier': 'ecosystem',
          'operator-tier-id': '4',
          'operator-kind-id': '21',
        },
        tags: ['rust', 'tui', 'cli', 'ratatui'],
      },
      spec: {
        type: 'cli-devtool',
        lifecycle: 'production',
        owner: 'team-platform',
      },
    },
    // Engines tier - Internal Tooling (ID 16)
    {
      apiVersion: 'backstage.io/v1alpha1',
      kind: 'Component',
      metadata: {
        name: 'backstage-server',
        namespace: 'default',
        title: 'Backstage Server',
        description: 'Bun-compiled Backstage server with embedded frontend',
        labels: {
          'operator-tier': 'engines',
          'operator-tier-id': '3',
          'operator-kind-id': '16',
        },
        tags: ['typescript', 'bun', 'backstage', 'hono'],
      },
      spec: {
        type: 'internal-tool',
        lifecycle: 'development',
        owner: 'team-platform',
      },
    },
    // Foundation tier - Infrastructure (ID 1)
    {
      apiVersion: 'backstage.io/v1alpha1',
      kind: 'Component',
      metadata: {
        name: 'infrastructure',
        namespace: 'default',
        title: 'Infrastructure (IaC)',
        description: 'Cloud resources and Terraform configurations',
        labels: {
          'operator-tier': 'foundation',
          'operator-tier-id': '1',
          'operator-kind-id': '1',
        },
        tags: ['terraform', 'aws', 'iac'],
      },
      spec: {
        type: 'infrastructure',
        lifecycle: 'production',
        owner: 'team-platform',
      },
    },
    // Standards tier - Software Library (ID 6)
    {
      apiVersion: 'backstage.io/v1alpha1',
      kind: 'Component',
      metadata: {
        name: 'shared-utils',
        namespace: 'default',
        title: 'Shared Utilities',
        description: 'Reusable internal logic packages and utilities',
        labels: {
          'operator-tier': 'standards',
          'operator-tier-id': '2',
          'operator-kind-id': '6',
        },
        tags: ['library', 'utils', 'shared'],
      },
      spec: {
        type: 'software-library',
        lifecycle: 'production',
        owner: 'team-platform',
      },
    },
    // Noncurrent tier - Reference/Example (ID 22)
    {
      apiVersion: 'backstage.io/v1alpha1',
      kind: 'Component',
      metadata: {
        name: 'examples',
        namespace: 'default',
        title: 'Example Projects',
        description: 'Best-practice implementation examples and tutorials',
        labels: {
          'operator-tier': 'noncurrent',
          'operator-tier-id': '5',
          'operator-kind-id': '22',
        },
        tags: ['examples', 'tutorials', 'reference'],
      },
      spec: {
        type: 'reference-example',
        lifecycle: 'experimental',
        owner: 'team-platform',
      },
    },
  ];

  for (const entity of sampleEntities) {
    catalogStorage.addEntity(entity);
    searchIndex.indexEntity(entity);
  }
  console.log('Added sample catalog entities with taxonomy tiers');
}

// MIME type mapping for static assets
function getMimeType(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase();
  const mimeTypes: Record<string, string> = {
    'html': 'text/html',
    'css': 'text/css',
    'js': 'application/javascript',
    'json': 'application/json',
    'png': 'image/png',
    'jpg': 'image/jpeg',
    'jpeg': 'image/jpeg',
    'gif': 'image/gif',
    'svg': 'image/svg+xml',
    'ico': 'image/x-icon',
    'woff': 'font/woff',
    'woff2': 'font/woff2',
    'ttf': 'font/ttf',
    'eot': 'application/vnd.ms-fontobject',
  };
  return mimeTypes[ext || ''] || 'application/octet-stream';
}

const app = new Hono();

// Enable CORS for Operator REST API integration
app.use('/*', cors({
  origin: ['http://localhost:7007', 'http://localhost:7008'],
  allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
}));

// Health check endpoint
app.get('/health', (c) => c.json({ status: 'ok', timestamp: new Date().toISOString() }));

// Status endpoint
app.get('/api/status', (c) => {
  const catalogStats = catalogStorage.getStats();
  const searchStats = searchIndex.getStats();
  return c.json({
    status: 'running',
    version: '1.0.0',
    mode: 'standalone',
    catalog: catalogStats,
    search: searchStats,
    branding: {
      appTitle: themeConfig.appTitle,
      orgName: themeConfig.orgName,
    },
  });
});

// Mount catalog API routes
const catalogRoutes = createCatalogRoutes(catalogStorage);
app.route('/api/catalog', catalogRoutes);

// Mount search API routes
const searchRoutes = createSearchRoutes(searchIndex);
app.route('/api/search', searchRoutes);

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

// Proxy route for Backstage proxy plugin convention (/api/proxy/operator/*)
// Used by OperatorApiClient and homepage widgets
app.all('/api/proxy/operator/*', async (c) => {
  const path = c.req.path.replace('/api/proxy/operator', '');
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

// Branding configuration endpoint - used by frontend for theming
app.get('/api/branding', (c) => c.json(themeConfig));

// Serve logo from branding directory
app.get('/branding/logo.svg', async (c) => {
  if (!themeConfig.logoPath) {
    return c.notFound();
  }

  const logoFullPath = join(brandingPath, themeConfig.logoPath);
  try {
    const logo = await readFile(logoFullPath);
    return new Response(logo, {
      headers: {
        'Content-Type': 'image/svg+xml',
        'Cache-Control': 'public, max-age=3600',
      },
    });
  } catch {
    return c.notFound();
  }
});

// Serve embedded static assets
app.get('/static/*', async (c) => {
  const path = c.req.path.slice(1); // Remove leading /
  const blob = assetMap.get(path);

  if (blob) {
    return new Response(blob, {
      headers: {
        'Content-Type': getMimeType(path),
        'Cache-Control': 'public, max-age=31536000, immutable',
      },
    });
  }

  return c.notFound();
});

// Fallback status page (shown when no frontend is embedded)
const statusPage = (apiUrl: string, stats: { entityCount: number; locationCount: number }) => `
<!DOCTYPE html>
<html>
<head>
  <title>Operator Backstage</title>
  <style>
    body { font-family: system-ui; max-width: 800px; margin: 50px auto; padding: 20px; }
    h1 { color: #333; }
    .status { padding: 20px; background: #f0f0f0; border-radius: 8px; margin-bottom: 20px; }
    code { background: #e0e0e0; padding: 2px 6px; border-radius: 4px; }
    .grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; }
    .stat { background: #e8f4f8; padding: 15px; border-radius: 8px; text-align: center; }
    .stat-value { font-size: 2em; font-weight: bold; color: #0066cc; }
  </style>
</head>
<body>
  <h1>Operator Backstage Server</h1>
  <div class="status">
    <p><strong>Status:</strong> Running</p>
    <p><strong>Mode:</strong> Standalone (compiled binary)</p>
    <p><strong>Operator API:</strong> <code>${apiUrl}</code></p>
    <p><strong>Frontend:</strong> Not embedded (build with <code>bun run build</code>)</p>
  </div>
  <div class="grid">
    <div class="stat">
      <div class="stat-value">${stats.entityCount}</div>
      <div>Catalog Entities</div>
    </div>
    <div class="stat">
      <div class="stat-value">${stats.locationCount}</div>
      <div>Locations</div>
    </div>
  </div>
  <h2>Available Endpoints</h2>
  <ul>
    <li><code>GET /health</code> - Health check</li>
    <li><code>GET /api/status</code> - Server status</li>
    <li><code>GET /api/catalog/entities</code> - List catalog entities</li>
    <li><code>GET /api/catalog/entities/by-query</code> - Query entities</li>
    <li><code>POST /api/search/query</code> - Search entities</li>
    <li><code>GET /api/issuetypes</code> - List issue types</li>
    <li><code>GET /api/collections</code> - List collections</li>
    <li><code>ALL /api/operator/*</code> - Proxy to Operator REST API</li>
  </ul>
</body>
</html>
`;

// SPA fallback - serve index.html for all non-API routes
app.get('*', async (c) => {
  // If we have embedded frontend, serve index.html for SPA routing
  if (hasEmbeddedFrontend) {
    const indexBlob = assetMap.get('index.html');
    if (indexBlob) {
      return c.html(await indexBlob.text());
    }
  }

  // Fallback to status page if no frontend embedded
  const stats = catalogStorage.getStats();
  return c.html(statusPage(operatorApiUrl, stats));
});

// Start server
const port = parseInt(process.env.PORT || '7007');
const catalogStats = catalogStorage.getStats();
console.log(`Operator Backstage Server starting on port ${port}...`);
console.log(`Operator API: ${operatorApiUrl}`);
console.log(`Catalog: ${catalogStats.entityCount} entities, ${catalogStats.locationCount} locations`);
console.log(`Embedded assets: ${assetMap.size} files${hasEmbeddedFrontend ? ' (frontend ready)' : ' (no frontend)'}`);
console.log(`Persistence: ${catalogPersistPath}`);
console.log(`Branding: "${themeConfig.appTitle}" (${themeConfig.orgName})`);

export default {
  port,
  fetch: app.fetch,
};
