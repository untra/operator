# Registry mirror prefix for the base images, e.g. registry.example/org/mirror/; empty pulls from upstream.
ARG BASE_REGISTRY=""

FROM ${BASE_REGISTRY}oven/bun:1.3.14 AS bun

# Tag tracks @playwright/test in package.json; the image ships the matching browsers and node.
FROM ${BASE_REGISTRY}mcr.microsoft.com/playwright:v1.63.0-noble

COPY --from=bun /usr/local/bin/bun /usr/local/bin/bun
WORKDIR /e2e
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile
COPY . .

ENTRYPOINT ["npx", "playwright", "test"]
