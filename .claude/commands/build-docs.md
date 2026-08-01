---
description: Build and serve the docs site locally with Jekyll
allowed-tools: Bash, Read
model: sonnet
---

# Build Docs

Build and serve the `docs/` subproject locally for inspection. Stop immediately on any failure and report the error.

## Workflow

Run each step sequentially from the repo root. If any step fails, stop and report the failure clearly.

1. **Install Ruby dependencies**: `cd docs && bundle install`
2. **Build the full site**: `make docs` from the repo root. This runs the whole pipeline in order — ts-rs bindings, the generated reference docs and hosted collection bundle (`cargo run -- docs`), the shared `webcomponents/` bundle, the copy into `docs/assets/js/`, then Jekyll.
3. **Serve locally**: `cd docs && bundle exec jekyll serve` (run in background so the session remains interactive; serves on port 4000)
4. **Report**: Confirm the site is running at http://localhost:4000. Let the user know it auto-rebuilds on file changes.

## Notes

- Use `make docs`, not a bare `bundle exec jekyll build`. Jekyll alone will not regenerate the collection bundle or build the web components, so `/workflows/` pages render an empty graph canvas.
- `bun` is required for step 2 (the `webcomponents/` build). `docs/assets/js/` is a build artifact and is gitignored.
- If port 4000 is already in use, report the conflict and suggest killing the existing process or using `--port` to pick a different one.
- To stop the server later, kill the background Jekyll process.
