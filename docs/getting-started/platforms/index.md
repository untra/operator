---
title: "Supported Workspace Platforms"
description: "Workspace platform integrations for running Operator in remote development environments."
layout: doc
---

# Supported Workspace Platforms

Operator can run as a background service in remote workspace platforms, providing API access and dashboard visibility without requiring a local terminal.

## Available Options

| Option | Status | Notes |
|--------|--------|-------|
| [Coder](/getting-started/platforms/coder/) | Supported | Terraform module, runs Operator as background API server with dashboard |
| [Docker](/getting-started/platforms/docker/) | Supported | Official multi-arch image (`untra/operator`); container is the workspace, mount your projects root at `/op` |
