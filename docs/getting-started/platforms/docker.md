---
title: "Docker"
description: "Run Operator from an official multi-arch Docker image, mounting your projects root as the workspace."
layout: doc
---

<span class="badge supported">Supported</span>

Run [Operator](https://operator.untra.io) from an official multi-arch container image. The image bundles the Operator binary (with the embedded web dashboard and REST API) and the `opr8r` client on a slim Debian base, plus the `git` and `tmux` substrate Operator needs to launch agents. Mount your projects root into the container and Operator treats it as the workspace.

**Image:** [`untra/operator`](https://hub.docker.com/r/untra/operator) - `linux/amd64` and `linux/arm64`.


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
docker run --rm -v $(pwd):/op:rw \
  -e OPERATOR_REST_API__HOST=0.0.0.0 \
  -e OPERATOR_BOOTSTRAP_PASSWORD_FILE=/run/secrets/bootstrap \
  -p 127.0.0.1:7008:7008 untra/operator api
```

**`OPERATOR_REST_API__HOST=0.0.0.0` is required to publish the port at all.**

Operator binds `127.0.0.1` by default, which inside a container means the *container's* loopback - unreachable from the host no matter how you publish it.
Setting the bind address to `0.0.0.0` makes it reachable from the container network; `-p 127.0.0.1:7008:7008` then restricts which host interface it appears on. Both halves are needed, and they do different jobs.

Outside a container the default is unchanged: Operator binds loopback, and you do not need to set this.

**`-p 127.0.0.1:7008:7008` binds the published port to host loopback only.** A
bare `-p 7008:7008` publishes on every host interface.

> **Authentication.** The REST API is authenticated. On first start Operator has
> no admin account and only the bootstrap, login, and probe endpoints respond;
> visit `/setup` to set the admin password, or mount a bootstrap password so the
> account cannot be claimed by whoever reaches the port first. CORS defaults to
> same-origin; set `[rest_api].cors_origins` to allow specific origins. See
> [Authentication](/security/authentication/).

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
| `opr8r` binary | Client agent sessions call to report step completion for multi-step workflows |
| `git` | Branch and commit operations for ticket work |
| `tmux` | Default session wrapper Operator uses to spawn agent sessions |
| `ca-certificates` | TLS for LLM, kanban, and git provider APIs |

**Not included: the LLM CLI and its auth.** Operator launches agents via an LLM tool
(`claude`, `codex`, or `gemini`) that you supply. Two ways to provide it:

1. **Derived image** - extend the official image with your tool of choice:

   ```dockerfile
   FROM untra/operator
   # The base image defaults to the non-root `operator` user; switch to root to
   # install, then drop back.
   USER root
   RUN apt-get update && apt-get install -y --no-install-recommends nodejs npm \
    && npm install -g @anthropic-ai/claude-code \
    && rm -rf /var/lib/apt/lists/*
   USER 10001
   ```

2. **Mount + env vars** - mount an already-installed, authenticated CLI from the host
   and pass credentials. The container runs as uid/gid 10001 with `$HOME=/home/operator`:

   ```bash
   docker run --rm -v $(pwd):/op:rw \
     -v $HOME/.claude:/home/operator/.claude \
     -e ANTHROPIC_API_KEY \
     -it untra/operator
   ```

## Implied startup

Operator reads `.tickets/operator/config.toml` relative to its working directory. Because
the image uses `WORKDIR /op` and you mount your projects root at `/op`, an existing config
is picked up automatically - no flags required. Run from a directory without one and
Operator uses its built-in defaults.

A global override at `~/.config/operator/config.toml` (i.e.
`/home/operator/.config/operator/config.toml` in the container) also works if you mount it.

## Prerequisites

- Docker (with `buildx` for multi-arch hosts, which is the default on modern Docker).
- The host directory you mount at `/op` should be your **projects root** - the directory
  containing your code repositories and `.tickets/` - so Operator can start work in the
  right place.
- Use `-it` for the interactive TUI; omit it for one-shot subcommands and `api`.

## Troubleshooting

### Permission denied writing to the mounted directory

The container runs as the unprivileged `operator` user (uid/gid 10001), so it can only write
into `/op` (state, prompts, logs) if the mounted host directory is writable by uid 10001.
Either make the directory writable (e.g. `chown -R 10001:10001`, or `chmod -R g+w` with a
matching group) or run the container as your own uid:

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
