# E2E Testing Guide

This directory contains end-to-end tests for the Backstage server using Playwright.

## Overview

The tests verify:
- **Health endpoints**: `/health` and `/api/status`
- **Catalog page**: Page loads, headers, view toggle functionality
- **Issue Types page**: Page loads, headers, action buttons, navigation
- **Plugins page**: Page loads, plugin table, metadata display

## Running Tests

### Local Development

```bash
# Build and run tests with binary (recommended)
bun run build
USE_BINARY=true bun run test:e2e

# Or run from source (requires frontend build)
bun run build:frontend
bun run build:embeds
bun run test:e2e
```

### Quick Commands

```bash
# Full build and test
bun run build && USE_BINARY=true bun run test:e2e

# Run specific test file
USE_BINARY=true bunx playwright test e2e/catalog.spec.ts

# Run with UI mode for debugging
USE_BINARY=true bunx playwright test --ui

# Show test report
bunx playwright show-report
```

## CI Configuration

The CI workflow (`.github/workflows/backstage.yaml`) runs e2e tests as follows:

1. **Build job**: Compiles the binary with `bun run build`
2. **E2E job**: Downloads the binary artifact and runs tests with `USE_BINARY=true`

The `playwright.config.ts` webServer command:
```typescript
command: process.env.USE_BINARY === 'true'
  ? './dist/backstage-server'
  : 'bun run start',
```

## How the Build System Works

```
bun run build:frontend   → builds packages/app/dist/ (React app)
bun run build:embeds     → generates src/embedded-assets.ts
bun run build:standalone → compiles binary with embedded frontend
```

The full `bun run build` runs all three steps.

## Authentication

### How Guest Auth Works

The Backstage frontend uses a guest authentication provider that requires backend support. When a user clicks "Enter" on the login page, the frontend makes API calls to authenticate and obtain a session token.

The standalone Hono server (`src/standalone.ts`) mocks these Backstage auth endpoints:

| Endpoint | Purpose |
|----------|---------|
| `GET /api/auth` | Service availability check |
| `GET /api/auth/providers` | Lists available auth providers |
| `GET /api/auth/guest` | Guest provider info |
| `GET /api/auth/guest/users` | Lists available guest users |
| `GET/POST /api/auth/guest/start` | Initiates guest sign-in, returns token |
| `GET/POST /api/auth/guest/refresh` | Refreshes session token |
| `GET /api/auth/.well-known/openid-configuration` | OpenID discovery |
| `GET /api/auth/.well-known/jwks.json` | JSON Web Key Set |

The mock endpoints return a simple base64-encoded token that satisfies the frontend's auth requirements. This token includes:
- `sub`: User entity reference (`user:development/guest`)
- `ent`: Ownership entity refs
- `iat`/`exp`: Token timestamps

### How Auth is Tested in Playwright

Since Backstage guest auth stores identity in memory (not cookies or localStorage), authentication doesn't persist across page navigations in Playwright. Each test handles this with the `gotoWithAuth()` helper:

```typescript
// e2e/auth.ts
export async function gotoWithAuth(page: Page, path: string): Promise<void> {
  await page.goto(path);
  await loginAsGuest(page);
}

export async function loginAsGuest(page: Page): Promise<void> {
  const enterButton = page.getByRole('button', { name: 'Enter' });
  try {
    await enterButton.waitFor({ state: 'visible', timeout: 3000 });
    await enterButton.click();
    await expect(enterButton).not.toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);
  } catch {
    // Not on login page - already authenticated
  }
}
```

**How it works:**
1. Navigate to the target page (e.g., `/catalog`)
2. Check if the "Enter" button (guest login) is visible
3. If visible, click it and wait for login to complete
4. If not visible, assume already authenticated and continue

**Why this approach:**
- Backstage stores auth state in React context (memory), not browser storage
- Each Playwright test gets a fresh browser context
- Global setup with `storageState` doesn't work because there's nothing to persist
- Per-navigation auth handling ensures each test can authenticate reliably

### Test Usage

Tests use `gotoWithAuth()` instead of `page.goto()`:

```typescript
test('displays Repositories header', async ({ page }) => {
  await gotoWithAuth(page, '/catalog');  // Handles auth automatically
  await expect(page.getByRole('heading', { name: 'Repositories' })).toBeVisible();
});
```

### Debugging Auth Issues

If tests fail with auth errors:

1. **Check the login page appears**: The test should see the "Enter" button
2. **Check the mock endpoints**: Server logs will show auth API calls
3. **Check the token format**: Frontend expects specific response structure
4. **Check timing**: Login may need more time to complete (adjust timeouts)

To debug, run with headed mode:
```bash
USE_BINARY=true bunx playwright test --headed --debug
```

## Test Structure

```
e2e/
├── auth.ts           # Authentication helper (gotoWithAuth)
├── health.spec.ts    # API health check tests
├── catalog.spec.ts   # Catalog page tests
├── issuetypes.spec.ts # Issue Types page tests
├── plugins.spec.ts   # Plugins page tests
└── README.md         # This file
```

## Troubleshooting

### Tests fail with "element not found"

1. **Check if frontend is embedded**: Run the binary and visit http://localhost:7007/
   - If you see a status page instead of the React app, the frontend isn't embedded
   - Rebuild with `bun run build`

2. **Verify assets exist**:
   ```bash
   ls -la src/assets/           # Should have static files
   ls -la packages/app/dist/    # Should have built React app
   cat src/embedded-assets.ts   # Should import from ./assets/
   ```

### Tests fail with auth errors

The standalone server mocks Backstage's guest auth. If auth fails:

1. Check the server logs for auth endpoint calls
2. Verify the frontend is using the expected auth flow
3. The mock endpoints are in `src/standalone.ts`

### Tests fail in CI

1. **Check artifact download**: Ensure the binary artifact is downloaded correctly
2. **Check executable permission**: Binary needs `chmod +x`
3. **Check environment variable**: `USE_BINARY=true` must be set

## Key Files

| File | Purpose |
|------|---------|
| `playwright.config.ts` | Test configuration, webServer command |
| `src/standalone.ts` | Standalone server with auth endpoints |
| `src/embedded-assets.ts` | Generated file with frontend imports |
| `packages/app/dist/` | Built React frontend |
| `.github/workflows/backstage.yaml` | CI workflow |

## Expected Test Behavior

### Health tests (no auth required):
- `GET /health` → `{ "status": "ok" }`
- `GET /api/status` → `{ "status": "running", "mode": "standalone" }`

### UI tests (require guest auth):
- `/catalog` → Repositories heading, view toggle
- `/issuetypes` → Issue Types heading, Create/Collections buttons
- `/plugins` → Installed Plugins table with catalog, search, issuetypes
