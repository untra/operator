---
description: Build, lint, test, and run the backstage-server locally
allowed-tools: Bash, Read
model: sonnet
---

# Build Backstage Server

Build, validate, and run the `backstage-server` subproject locally for inspection. Stop immediately on any failure and report the error.

## Workflow

Run each step sequentially from the `backstage-server/` directory. If any step fails, stop and report the failure clearly.

1. **Install dependencies**: `cd backstage-server && bun install`
2. **Lint**: `cd backstage-server && bun run lint`
3. **Typecheck**: `cd backstage-server && bun run typecheck`
4. **Test**: `cd backstage-server && bun test`
5. **Build**: `cd backstage-server && bun run build` (builds frontend, embeds assets, produces standalone binary at `dist/backstage-server`)
6. **Verify binary**: `ls -lh backstage-server/dist/backstage-server` (confirm binary exists and report its size)
7. **Run dev server**: `cd backstage-server && bun run dev` (run in background with hot-reload on port 7007)
8. **Report**: Confirm the server is running at http://localhost:7007. Note that it proxies to the Operator API at :7008.

## Notes

- If port 7007 is already in use, report the conflict and suggest killing the existing process.
- To stop the dev server later, kill the background Bun process.
