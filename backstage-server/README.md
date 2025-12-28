# Backstage Server

A Bun-compiled standalone web portal for Operator, built with Backstage components and Hono.

## Overview

This is a self-contained Backstage-based web portal that provides:

- **Dashboard**: Queue status, active agents, issue types overview
- **Catalog**: Software catalog with Operator's 5-tier taxonomy
- **Issue Types**: Create and manage issue type templates
- **Plugins**: View installed plugins

The server compiles to a single binary using Bun with embedded frontend assets.

## Tech Stack

- **Runtime**: Bun (compiled binary)
- **Backend**: Hono web framework
- **Frontend**: React 18 + Backstage UI components
- **Language**: TypeScript 5
- **Testing**: Bun test (unit) + Playwright (E2E)

## Development

### Prerequisites

- Bun 1.0+
- Node.js 18+ (for Backstage CLI)

### Commands

```bash
# Development server with hot reload
bun run dev

# Build production binary
bun run build

# Run the built binary
./dist/backstage-server
```

### Testing

```bash
# Unit tests
bun test

# E2E tests (requires server running)
bun run test:e2e

# E2E with UI
bun run test:e2e:ui

# E2E headed mode
bun run test:e2e:headed
```

### Linting & Type Checking

```bash
# Lint all packages
bun run lint

# Auto-fix lint issues
bun run lint:fix

# Type check
bun run typecheck

# Dependency analysis
bun run knip
```

## Quality Enforcement

This project uses multiple tools to enforce code quality:

| Tool | Purpose | Command |
|------|---------|---------|
| ESLint | Code linting | `bun run lint` |
| TypeScript | Type checking | `bun run typecheck` |
| Bun test | Unit tests | `bun test` |
| Playwright | E2E tests | `bun run test:e2e` |
| Knip | Unused exports/deps | `bun run knip` |

### CI Checks

All PRs must pass:

1. `bun run lint` - No lint errors
2. `bun run typecheck` - No type errors
3. `bun test` - All unit tests pass
4. `bun run knip` - No unused exports or dependencies
5. `bun run test:e2e` - All E2E tests pass

## Architecture

```
backstage-server/
├── src/
│   ├── standalone.ts      # Hono server entry point
│   ├── embedded-assets.ts # Auto-generated asset embeddings
│   ├── catalog/           # Catalog storage and routes
│   └── search/            # Search index
├── packages/
│   ├── app/               # React frontend
│   │   ├── src/
│   │   │   ├── AppNew.tsx        # New frontend system
│   │   │   ├── App.tsx           # Legacy routing
│   │   │   ├── components/       # UI components
│   │   │   │   ├── home/         # Dashboard widgets
│   │   │   │   ├── catalog/      # Catalog views
│   │   │   │   └── plugins/      # Plugins page
│   │   │   └── extensions/       # Backstage extensions
│   │   └── public/               # Static assets
│   ├── backend/           # Backstage backend (reference)
│   └── plugins/
│       └── plugin-issuetypes/    # Issue types plugin
├── e2e/                   # Playwright E2E tests
├── scripts/
│   └── generate-embeds.ts # Asset embedding generator
├── bunfig.toml            # Bun configuration
├── knip.json              # Knip configuration
└── playwright.config.ts   # Playwright configuration
```

## Build Process

1. **Frontend build**: `backstage-cli package build` compiles React app
2. **Asset embedding**: `generate-embeds.ts` creates `embedded-assets.ts`
3. **Binary compilation**: Bun compiles server + assets into single binary

```bash
bun run build:frontend   # Build React app
bun run build:embeds     # Generate asset embeddings
bun run build:standalone # Compile to binary
```

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Health check |
| `GET /api/status` | Server status + catalog stats |
| `GET /api/catalog/entities` | List catalog entities |
| `POST /api/search/query` | Search catalog |
| `GET /api/issuetypes` | List issue types (proxied to Operator) |
| `ALL /api/operator/*` | Proxy to Operator REST API |

## Configuration

The server reads configuration from:

- `~/.operator/backstage-catalog.json` - Catalog persistence
- `~/.operator/backstage/branding/theme.json` - Custom theming

Environment variables:

- `OPERATOR_API_URL` - Operator REST API URL (default: `http://localhost:7008`)
- `PORT` - Server port (default: `7007`)
