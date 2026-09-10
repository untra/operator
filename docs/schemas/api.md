---
title: "REST API"
description: "Interactive Swagger UI documentation for the Operator! REST API for issue type management."
layout: doc
---

Interactive API documentation powered by Swagger UI.

The Operator REST API provides endpoints for managing issue types and collections programmatically.

## Quick Links

- **Base URL**: `http://localhost:7008/api/v1`
- **Liveness probe**: `GET /livez` (public)
- **Readiness probe**: `GET /readyz` (public)
- **Health Check**: `GET /api/v1/health` (requires `read`)
- **Status**: `GET /api/v1/status` (requires `read`)

## Authentication

Every endpoint below is authenticated except the probes, the bootstrap and
login endpoints, and the OAuth device and token endpoints. Two schemes are
accepted:

| Scheme | Used by | Notes |
|--------|---------|-------|
| `bearerAuth` | CLI, IDE clients, integrations, MCP | `Authorization: Bearer <access token>`. Access tokens are short-lived; obtain one at the token endpoint with a refresh token or a service access key. |
| `sessionCookie` | The web dashboard | Opaque `__Host-operator_session` cookie. Cookie-authenticated mutations additionally require a CSRF token and a matching `Origin`. |

Each operation declares the scope it requires - `read`, `write`, `execute`, or `admin`. A credential without the scope receives `403`; no credential at all receives `401`.

Full details, including bootstrap, the device flow, and access-key lifecycle,
are in [Authentication](/security/authentication/).

## Starting the API Server

```bash
# Start with default port (7008)
cargo run -- api

# Start with custom port
cargo run -- api --port 8080
```

## Interactive Documentation

<div id="swagger-ui"></div>

<link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
<script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>
window.onload = () => {
  SwaggerUIBundle({
    url: "{{ '/schemas/openapi.json' | relative_url }}",
    dom_id: '#swagger-ui',
    presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset],
    layout: "BaseLayout",
    deepLinking: true,
    showExtensions: true,
    showCommonExtensions: true
  });
};
</script>

## Regenerating the Spec

The OpenAPI specification is auto-generated from source code annotations:

```bash
# Generate just the OpenAPI spec
cargo run -- docs --only openapi

# Generate all documentation
cargo run -- docs
```

The spec is written to `docs/schemas/openapi.json`.
