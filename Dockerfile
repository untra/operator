# glibc 2.41 >= the ubuntu-24.04 build runners' 2.39, so the GNU binary runs.
FROM debian:trixie-slim

# Populated automatically by buildx per target platform (amd64 / arm64).
ARG TARGETARCH

# Substrate Operator needs to launch agents: git (VCS ops), tmux (session
# wrapper), ca-certificates (TLS to LLM/kanban APIs). The LLM CLI (claude /
# codex / gemini) and its auth are supplied by the user via a derived image or
# a mount + env vars -- not baked in here.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates git tmux \
 && rm -rf /var/lib/apt/lists/*

# CI stages the prebuilt release binary as operator-linux-${TARGETARCH}
# (operator-linux-x86_64 is renamed to operator-linux-amd64; arm64 matches).
COPY operator-linux-${TARGETARCH} /usr/local/bin/operator
RUN chmod +x /usr/local/bin/operator

# Fail the multi-arch build (incl. arm64 under QEMU binfmt from
# setup-qemu-action) before push if the binary can't execute on this base.
# --version short-circuits in clap before any config or tmux load.
RUN ["/usr/local/bin/operator", "--version"]

# Run as an unprivileged user with a writable HOME by default. A compromised
# agent tool then can't act as root against the mounted workspace.
# Debian ships a legacy `operator` system group, so assign the existing `users`
# group rather than letting useradd create a colliding same-name group.
RUN useradd --create-home --uid 1000 --gid users operator
USER operator

# Mount your projects root here: `docker run -v $(pwd):/op:rw ...`.
# Operator auto-loads .tickets/operator/config.toml relative to the cwd.
# Created owned by uid 1000, so the default (no-mount) workdir is writable.
WORKDIR /op
ENTRYPOINT ["operator"]
