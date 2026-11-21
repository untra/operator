---
title: "Operator API"
description: "Reach Operator's REST API, Swagger UI, and OpenAPI spec from the Connections page."
layout: doc
---

The **Operator API** card shows the profile-scoped API base for the active host, including HTTPS and any configured path prefix.

- **Check connection** calls the API and reports its latency and the time of the check.
- **Copy API base** copies the profile-scoped base URL.
- **Swagger** and **OpenAPI** open the interactive docs and the raw spec on the same host.

Swagger and OpenAPI need a signed-in browser. If either route is blocked, check that your ingress forwards `/swagger-ui/` and `/api-docs/`.

For endpoint details, see the [REST API schema](/schemas/api/).
