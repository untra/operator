---
title: "LM Studio"
description: "Use a Windows personal computer for local inference from Operator in Kubernetes."
layout: doc
---

<span class="badge proto">Proto</span>

Run your model on a Windows x64 personal computer or gaming machine.
Model listing alone does not verify agent compatibility.

```text
VS Code on your workstation -- HTTPS --> Operator in Kubernetes
                                            |
                                      opr8r + agent CLI
                                            |
                                      LAN HTTP inference
                                            v
                                   Windows PC + LM Studio
```

`opr8r` is a step wrapper: it runs the CLI, tracks its exit, and reports progress to Operator.

## Prepare the PC

1. Install [LM Studio](https://lmstudio.ai/) and download a tool-capable model that fits your GPU memory.
   Use the exact model ID returned by `/v1/models`.
   Context size also consumes memory; LM Studio recommends more than roughly 25k context for its coding-agent integrations.
   Start with one ticket at a time.
2. Start the API server on port `1234`. Enable **Serve on Local Network** and **Require Authentication**, then create an LM Studio API token.
   See [server settings](https://lmstudio.ai/docs/developer/core/server/settings). CORS and LM Studio's own MCP integrations are unnecessary here.
3. Reserve a stable LAN IP for the PC in your router, for example `192.168.1.50`.
4. In **Windows Defender Firewall**, allow inbound TCP `1234` on the Private profile only from the cluster's actual source addresses. Depending on your CNI, the PC may see node addresses rather than pod addresses. An administrator can use:

   ```powershell
   New-NetFirewallRule -DisplayName "LM Studio from Operator" -Direction Inbound -Action Allow -Protocol TCP -LocalPort 1234 -Profile Private -RemoteAddress <cluster-source-CIDR>
   ```

5. Keep the PC awake during jobs and confirm the server and model are available after a Windows restart. Do not forward port `1234` from the Internet. Use a trusted LAN for HTTP; the token is not encrypted in transit. For an untrusted network, provide a trusted HTTPS endpoint or encrypted tunnel.

## Prepare Operator's pod

The stock image contains `opr8r`, `tmux`, and Git, but neither agent CLI. Build an arm64 derived image with a supported Node.js runtime and both CLIs. Pin the three build arguments to the versions you will record in the acceptance run:

```dockerfile
ARG NODE_VERSION
ARG OPERATOR_VERSION
FROM node:${NODE_VERSION}-bookworm-slim AS node
FROM untra/operator:${OPERATOR_VERSION}
USER root
COPY --from=node /usr/local/ /usr/local/
ARG CODEX_VERSION
ARG CLAUDE_VERSION
RUN npm install -g @openai/codex@${CODEX_VERSION} @anthropic-ai/claude-code@${CLAUDE_VERSION}
USER 10001
```

Build
with `--platform linux/arm64`, publish to your registry, and select that image with the chart's `image.repository` and `image.tag` values. Verify both CLI versions from the running pod before configuring delegators.

Store the LM Studio token in a Kubernetes Secret as `LM_STUDIO_API_TOKEN` and reference that Secret with `extraEnvFrom`. Do not put the token in values files, Operator configuration, or checked-in command examples.

```yaml
publicUrl: https://operator.example.com
extraEnvFrom:
  - secretRef:
      name: operator-lmstudio
egress:
  allowPrivateCidrs:
    - 192.168.1.50/32
networkPolicy:
  enabled: true
  egress:
    allowDNS: true
    additionalRules:
      - to:
          - ipBlock:
              cidr: 192.168.1.50/32
        ports:
          - protocol: TCP
            port: 1234
```

Merge this with the egress rules needed for your Git and other services. The application allowlist does not create a route or open a firewall. NetworkPolicy also requires a CNI that enforces it.

Verify connectivity **from the pod**, not only from your workstation:

```bash
kubectl -n operator exec statefulset/operator -- sh -c \
  'curl --fail --silent --show-error -H "Authorization: Bearer $LM_STUDIO_API_TOKEN" http://192.168.1.50:1234/v1/models'
```

A successful curl verifies the network path and token; Operator's own model probe separately verifies its application egress policy.

## Configure one server and two delegators

Edit `/op/.tickets/operator/config.toml` in the persistent workspace, or use Operator's model-server and delegator interfaces. Replace `MODEL_ID` below with an exact ID from the server. Both delegators share one endpoint and token:

```toml
[[model_servers]]
name = "gaming-pc"
kind = "lmstudio"
base_url = "http://192.168.1.50:1234"
api_key_env = "LM_STUDIO_API_TOKEN"
display_name = "Gaming PC"

[[delegators]]
name = "gaming-codex"
llm_tool = "codex"
model = "MODEL_ID"
model_server = "gaming-pc"

[[delegators]]
name = "gaming-claude"
llm_tool = "claude"
model = "MODEL_ID"
model_server = "gaming-pc"
```

Outside Helm, configure the equivalent application policy with:

```toml
[egress]
allow_private_cidrs = ["192.168.1.50/32"]
```

The environment equivalent is `OPERATOR_EGRESS__ALLOW_PRIVATE_CIDRS=192.168.1.50/32,fd12::50/128`.
Restart Operator after changing deployment environment variables or configuration on disk. No Windows execution target is needed.

Operator selects tool-specific settings for `kind = "lmstudio"`:

- **Codex:** a per-launch custom provider uses `/v1/responses`, and your token reference. See [Codex providers](https://learn.chatgpt.com/docs/config-file/config-advanced) and [LM Studio's Codex integration](https://lmstudio.ai/docs/integrations/codex).
- **Claude Code:** `ANTHROPIC_BASE_URL` uses the server root, `ANTHROPIC_AUTH_TOKEN` references your token, and attribution headers are disabled. LM Studio serves `/v1/messages`; no protocol bridge is needed. See [LM Studio's Claude integration](https://lmstudio.ai/docs/integrations/claude-code).

The root URL and a trailing `/v1` form are both accepted. `extra_env` overrides derived environment settings. Without `api_key_env`, LM Studio probes do not reuse cloud-provider credentials; unauthenticated servers are supported, but this LAN walkthrough uses authentication.

## Drive it from VS Code

Set `operator.connectionMode` to `remote` and `operator.apiUrl` to your HTTPS Operator origin. Complete the extension's Operator authentication flow. This connection uses Operator credentials, not your LM Studio token.

Expand `gaming-pc` in the model-server tree, choose one of the two delegators for a small ticket, and watch its progress. Your IDE does not need direct access to LM Studio. Agent tools run against the repository in the pod's workspace.

## Acceptance checklist

Record Windows, LM Studio, Operator, both CLI versions, GPU/VRAM, exact model ID, and context settings with your results. No hardware combination is certified by this guide yet.

- Authenticated Operator model discovery lists the selected model.
- Each delegator completes a small ticket that reads a file, changes it, and runs a check. Confirm tool execution in Linux and inference in LM Studio logs.
- Step completion reaches Operator through `opr8r` and appears in VS Code.
- Native judge inference is currently unavailable: the repository has paused
  the `native-llm` Cargo feature. Judge configuration falls back to deterministic selection. Re-enabling and validating that feature is a separate prerequisite for testing native judgments against LM Studio.
- Stop LM Studio during a job: the job must report failure, without false completion or silently calling a cloud provider. Restore it and retry.

Promote this integration to Alpha only after recording successful runs for both CLIs. A model can support chat or listing while failing tool calls, streaming, or the context requirements of an agent.

## Troubleshooting

| Symptom | Check |
|---|---|
| Timeout or connection refused | PC sleep, server listening on LAN, Windows firewall, pod routing, CNI policy |
| Private destination rejected | The PC's actual address is in `egress.allow_private_cidrs`; retain `publicUrl` |
| 401 | Token exists in the pod and matches LM Studio; Operator API auth is separate |
| 404 on inference | LM Studio supports Responses for Codex and Messages for Claude; base URL has no duplicated `/v1` |
| Model lists but agent fails | Tool support, model ID, context allocation, GPU memory, CLI/server version compatibility |
| Long first response | Model cold loading or automatic unloading; inspect LM Studio logs |
| Stale running ticket | Inspect CLI output and `opr8r` callback connectivity before retrying |

The application egress policy protects Operator's model probes and native inference. It is not a sandbox for spawned agent processes; use NetworkPolicy and the execution environment to constrain their network access.

Codex fresh launches do not use Operator's tracking UUID as a Codex resume ID. Resuming a native Codex conversation requires its actual CLI session ID; this walkthrough validates fresh ticket steps rather than conversation resumption.
