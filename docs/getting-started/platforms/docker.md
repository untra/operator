---
title: "Docker"
description: "Run Operator from an official multi-arch Docker image, mounting your projects root as the workspace."
layout: doc
---

# Docker

<span class="badge supported">Supported</span>

Run [Operator](https://operator.untra.io) from an official multi-arch container image. The image bundles the Operator binary (with the embedded web dashboard and REST API) on a slim Debian base, plus the `git` and `tmux` substrate Operator needs to launch agents. Mount your projects root into the container and Operator treats it as the workspace.

**Image:** [`untra/operator`](https://hub.docker.com/r/untra/operator) — `linux/amd64` and `linux/arm64`.

> **Not to be confused with** the `[docker]` config section, which makes Operator launch each *agent* inside a container. This page is about distributing *Operator itself* as a container image. The two are orthogonal.

## Usage

Run the TUI dashboard from your projects root:

```bash
docker run --rm -v $(pwd):/op:rw -it untra/operator
```

The image sets `WORKDIR /op`, so the mounted directory becomes Operator's working
directory. If it contains `.tickets/operator/config.toml`, Operator loads it as implied
startup; otherwise it falls back to built-in defaults.

Subcommands are appended after the image name. Run as a background REST API service:

```bash
docker run --rm -v $(pwd):/op:rw -p 127.0.0.1:7008:7008 untra/operator api
```

> **Security:** the REST API is **unauthenticated**, sends permissive CORS headers, and
> exposes mutating endpoints (launching agents, editing config). The publish above binds it
> to loopback (`127.0.0.1`) only — a bare `-p 7008:7008` would expose it on every host
> interface. Do not publish it to untrusted networks; if you need remote access, put it
> behind an authenticating reverse proxy.

Any Operator subcommand works the same way:

```bash
docker run --rm -v $(pwd):/op:rw untra/operator queue     # show queue
docker run --rm -v $(pwd):/op:rw untra/operator setup     # initialize workspace
```

Pin a specific version instead of `latest`:

```bash
docker run --rm -v $(pwd):/op:rw -it untra/operator:{{ site.version }}
```

## What's in the image

| Included | Purpose |
|----------|---------|
| `operator` binary | The CLI/TUI/REST API, with the web dashboard embedded |
| `git` | Branch and commit operations for ticket work |
| `tmux` | Default session wrapper Operator uses to spawn agent sessions |
| `ca-certificates` | TLS for LLM, kanban, and git provider APIs |

**Not included: the LLM CLI and its auth.** Operator launches agents via an LLM tool
(`claude`, `codex`, or `gemini`) that you supply. Two ways to provide it:

1. **Derived image** — extend the official image with your tool of choice:

   ```dockerfile
   FROM untra/operator
   # The base image defaults to the non-root `operator` user; switch to root to
   # install, then drop back.
   USER root
   RUN apt-get update && apt-get install -y --no-install-recommends nodejs npm \
    && npm install -g @anthropic-ai/claude-code \
    && rm -rf /var/lib/apt/lists/*
   USER operator
   ```

2. **Mount + env vars** — mount an already-installed, authenticated CLI from the host
   and pass credentials. The container runs as uid 1000 with `$HOME=/home/operator`:

   ```bash
   docker run --rm -v $(pwd):/op:rw \
     -v $HOME/.claude:/home/operator/.claude \
     -e ANTHROPIC_API_KEY \
     -it untra/operator
   ```

## Implied startup

Operator reads `.tickets/operator/config.toml` relative to its working directory. Because
the image uses `WORKDIR /op` and you mount your projects root at `/op`, an existing config
is picked up automatically — no flags required. Run from a directory without one and
Operator uses its built-in defaults.

A global override at `~/.config/operator/config.toml` (i.e.
`/home/operator/.config/operator/config.toml` in the container) also works if you mount it.

## Prerequisites

- Docker (with `buildx` for multi-arch hosts, which is the default on modern Docker).
- The host directory you mount at `/op` should be your **projects root** — the directory
  containing your code repositories and `.tickets/` — so Operator can start work in the
  right place.
- Use `-it` for the interactive TUI; omit it for one-shot subcommands and `api`.

## Troubleshooting

### Permission denied writing to the mounted directory

The container runs as the unprivileged `operator` user (uid 1000), so it can only write into
`/op` (state, prompts, logs) if the mounted host directory is writable by uid 1000. On a
typical single-user Linux host your uid is already 1000 and it just works; otherwise either
make the directory writable (e.g. `chmod -R g+w` with a matching group, or `chown`) or run the
container as your own uid:

```bash
docker run --rm -v $(pwd):/op:rw --user $(id -u):$(id -g) -it untra/operator
```

When overriding `--user`, the chosen uid has no entry in the image, so point `$HOME` at a
writable location for any tool config (e.g. add `-e HOME=/op`).

### The dashboard renders as garbled output

The TUI needs an interactive TTY. Include `-it` in the `docker run` invocation.

### Agents fail to launch

The base image intentionally omits the LLM CLI. Confirm your derived image or mount
provides an authenticated `claude` / `codex` / `gemini` on `PATH` inside the container
(`docker run --rm --entrypoint sh untra/operator -c 'command -v claude'`).
