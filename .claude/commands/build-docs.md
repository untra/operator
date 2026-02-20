---
description: Build and serve the docs site locally with Jekyll
allowed-tools: Bash, Read
model: sonnet
---

# Build Docs

Build and serve the `docs/` subproject locally for inspection. Stop immediately on any failure and report the error.

## Workflow

Run each step sequentially from the `docs/` directory. If any step fails, stop and report the failure clearly.

1. **Install dependencies**: `cd docs && bundle install`
2. **Build site**: `cd docs && bundle exec jekyll build`
3. **Serve locally**: `cd docs && bundle exec jekyll serve` (run in background so the session remains interactive; serves on port 4000)
4. **Report**: Confirm the site is running at http://localhost:4000. Let the user know it auto-rebuilds on file changes.

## Notes

- If port 4000 is already in use, report the conflict and suggest killing the existing process or using `--port` to pick a different one.
- To stop the server later, kill the background Jekyll process.
