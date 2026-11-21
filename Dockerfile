# glibc 2.41 >= the ubuntu-24.04 build runners' 2.39, so the GNU binary runs.
FROM debian:trixie-slim@sha256:a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a

LABEL org.opencontainers.image.title="Operator" \
      org.opencontainers.image.description="Agent orchestration server and CLI" \
      org.opencontainers.image.url="https://operator.untra.io" \
      org.opencontainers.image.source="https://github.com/untra/operator" \
      org.opencontainers.image.licenses="MIT"

# Populated automatically by buildx per target platform (amd64 / arm64).
ARG TARGETARCH

# Substrate Operator needs to launch agents: git (VCS ops), tmux (session
# wrapper), ca-certificates (TLS to LLM/kanban APIs), openssh-client (every
# ssh and coder target launch, and git over SSH remotes), curl (in-pod reachability checks).
# The LLM CLI (claude / codex / gemini) and its auth are supplied by the user via a derived image or env vars
RUN apt-get update \
 && apt-get upgrade -y --no-install-recommends \
 && apt-get install -y --no-install-recommends ca-certificates curl git openssh-client tmux \
 && rm -rf /var/lib/apt/lists/*

# CI stages the prebuilt release binaries as {operator,opr8r}-linux-${TARGETARCH}
# (the -linux-x86_64 artifacts are renamed to -linux-amd64; arm64 matches).
COPY --chown=0:0 --chmod=0755 operator-linux-${TARGETARCH} /usr/local/bin/operator
COPY --chown=0:0 --chmod=0755 opr8r-linux-${TARGETARCH} /usr/local/bin/opr8r

# Fail the multi-arch build (incl. arm64 under QEMU binfmt from
# setup-qemu-action) before push if a binary can't execute on this base.
# --version short-circuits in clap before any config or tmux load.
RUN ["/usr/local/bin/operator", "--version"]
RUN ["/usr/local/bin/opr8r", "--version"]

# Run as an unprivileged user with a dedicated uid and gid.
RUN groupdel operator \
 && groupadd -g 10001 operator \
 && useradd -u 10001 -g 10001 -m -d /home/operator operator \
 && mkdir /op \
 && chown operator:operator /op
ENV HOME=/home/operator
USER operator

# Mount your projects root here: `docker run -v $(pwd):/op:rw ...`.
# Operator auto-loads .tickets/operator/config.toml relative to the cwd.
WORKDIR /op
EXPOSE 7008
ENTRYPOINT ["operator"]
