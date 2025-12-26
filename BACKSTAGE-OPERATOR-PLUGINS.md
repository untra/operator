# Backstage Operator Plugins

A Backstage plugin for managing Operator issue types and collections via the Operator REST API.

---

## Overview

The `@operator/plugin-issuetypes` plugin provides a web UI for managing issue type templates within Backstage. It communicates with the Operator REST API (port 7008) via Backstage's proxy middleware.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Backstage (port 7007)                         │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  @operator/plugin-issuetypes                                ││
│  │  - IssueTypesPage (list view)                               ││
│  │  - IssueTypeDetailPage (steps, fields)                      ││
│  │  - IssueTypeFormPage (create/edit)                          ││
│  │  - CollectionsPage (activate collections)                   ││
│  └──────────────────────┬──────────────────────────────────────┘│
│                         │ /api/proxy/operator                    │
│  ┌──────────────────────▼──────────────────────────────────────┐│
│  │  Backstage Proxy (app-config.yaml)                          ││
│  │  proxy: '/operator': target: http://localhost:7008          ││
│  └──────────────────────┬──────────────────────────────────────┘│
└─────────────────────────┼───────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────┐
│                 Operator REST API (port 7008)                    │
│  /api/v1/issuetypes, /api/v1/collections, /api/v1/status        │
└─────────────────────────────────────────────────────────────────┘
```

---

## File Structure

### Plugin Source (TypeScript)

```
plugins/
└── plugin-issuetypes/
    ├── package.json
    ├── tsconfig.json
    └── src/
        ├── index.ts              # Public exports
        ├── plugin.ts             # createPlugin, extensions
        ├── routes.ts             # RouteRefs
        ├── api/
        │   ├── index.ts
        │   ├── OperatorApi.ts    # API interface
        │   ├── OperatorApiClient.ts
        │   └── types.ts          # TypeScript types
        ├── components/
        │   ├── IssueTypesPage.tsx
        │   ├── IssueTypeDetailPage.tsx
        │   ├── IssueTypeFormPage.tsx
        │   └── CollectionsPage.tsx
        └── hooks/
            ├── useIssueTypes.ts
            └── useCollections.ts
```

### Generated Scaffold Output

When the Backstage scaffold is generated, the plugin is copied to:

```
.tickets/operator/backstage/
└── packages/
    └── plugins/
        └── plugin-issuetypes/
            └── (same structure as above)
```

---

## Pages

### Issue Types List (`/issuetypes`)

- Grid of issue type cards with glyph, name, mode badge
- Filter by source (builtin/user-defined)
- "Create Issue Type" button for new types
- Click to view details

### Issue Type Detail (`/issuetypes/:key`)

- Full metadata display (key, name, mode, branch prefix, etc.)
- Step list with prompts, outputs, permission modes
- Field definitions with types and options
- Edit/Delete buttons (disabled for builtin types)

### Issue Type Form (`/issuetypes/new`, `/issuetypes/:key/edit`)

- Create new or edit existing issue types
- Dynamic step editor (add/remove steps)
- Field type selector (string, enum, bool, date, text)
- Validation for key format and required fields

### Collections (`/issuetypes/collections`)

- List all collections with active indicator
- Activate button to switch active collection
- Confirmation dialog before activation

---

## API Integration

### Proxy Configuration

The plugin uses Backstage's proxy to reach the Operator REST API:

```yaml
# app-config.yaml
proxy:
  '/operator':
    target: 'http://localhost:7008'
    changeOrigin: true
```

### Endpoints Used

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/issuetypes` | GET | List all issue types |
| `/api/v1/issuetypes` | POST | Create new issue type |
| `/api/v1/issuetypes/:key` | GET | Get single issue type |
| `/api/v1/issuetypes/:key` | PUT | Update issue type |
| `/api/v1/issuetypes/:key` | DELETE | Delete issue type |
| `/api/v1/collections` | GET | List all collections |
| `/api/v1/collections/:name/activate` | PUT | Activate collection |

---

## Development

### Running Locally

1. Start the Operator REST API:
   ```bash
   cargo run -- api --port 7008
   ```

2. Generate the Backstage scaffold:
   ```bash
   cargo run -- scaffold
   ```

3. Start Backstage:
   ```bash
   cd .tickets/operator/backstage
   yarn install
   yarn dev
   ```

4. Open http://localhost:7007/issuetypes

### Plugin Development

The plugin source is in `plugins/plugin-issuetypes/`. After making changes:

1. Regenerate scaffold with `--force`:
   ```bash
   cargo run -- scaffold --force
   ```

2. Restart Backstage dev server

### Testing

```bash
# Rust validation
cargo fmt && cargo clippy -- -D warnings && cargo test

# TypeScript (in plugin directory)
cd plugins/plugin-issuetypes
yarn install
yarn lint
yarn build
```

---

## Scaffold Integration

The Backstage scaffold generator (`src/backstage/scaffold.rs`) has been updated to:

1. **Include plugins workspace** in `package.json`:
   ```json
   "workspaces": {
     "packages": ["packages/*", "packages/plugins/*"]
   }
   ```

2. **Add proxy configuration** to `app-config.yaml`:
   ```yaml
   proxy:
     '/operator':
       target: 'http://localhost:7008'
       changeOrigin: true
   ```

3. **Register plugin** in `App.tsx`:
   ```tsx
   import { issueTypesPlugin, IssueTypesPage } from '@operator/plugin-issuetypes';

   const app = createApp({
     plugins: [catalogPlugin, issueTypesPlugin],
   });
   ```

4. **Add proxy backend** to backend dependencies:
   ```json
   "@backstage/plugin-proxy-backend": "^0.5.0"
   ```

5. **Copy plugins directory** from `plugins/` to `packages/plugins/`

---

## TypeScript Types

Key interfaces (from `src/api/types.ts`):

```typescript
interface IssueTypeSummary {
  key: string;
  name: string;
  description: string;
  mode: 'autonomous' | 'paired';
  glyph: string;
  source: string;
  step_count: number;
}

interface StepResponse {
  name: string;
  display_name?: string;
  prompt: string;
  outputs: string[];
  allowed_tools: string[];
  requires_review: boolean;
  permission_mode: 'default' | 'plan' | 'acceptEdits' | 'delegate';
}

interface CollectionResponse {
  name: string;
  description: string;
  types: string[];
  is_active: boolean;
}
```

---

## Dependencies

### Plugin Dependencies

- `@backstage/core-components` - UI components
- `@backstage/core-plugin-api` - Plugin APIs
- `@material-ui/core` - Material UI
- `react-use` - React hooks utilities

### Backend Dependencies

- `@backstage/plugin-proxy-backend` - Proxy middleware

---

## Future Enhancements

- Step workflow visualization (diagram)
- Drag-and-drop step reordering
- Bulk operations for collections
- Import/export issue types as JSON
- Real-time updates via WebSocket
