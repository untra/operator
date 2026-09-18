---
title: "Supported Workspace Platforms"
description: "Workspace platform integrations for running Operator in remote development environments."
layout: doc
---

Operator can run as a background service in remote workspace platforms, providing API access and dashboard visibility without requiring a local terminal.

## Available Options

| Option | Status | Notes |
|--------|--------|-------|
| [Docker](/getting-started/platforms/docker/) | Supported | Official multi-arch image (`untra/operator`); container is the workspace, mount your projects root at `/op` |
| [Kubernetes](/getting-started/platforms/kubernetes/) | Alpha | OCI Helm chart; single-replica StatefulSet with persistent workspace, authenticated REST API and dashboard |

[Coder workspaces](/getting-started/remote-targets/coder/) are documented under [Remote Targets](/getting-started/remote-targets/). Running agents together with Operator in the same workspace remains local execution.
