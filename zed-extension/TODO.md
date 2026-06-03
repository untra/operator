# Zed Extension Status

Feature comparison and implementation status vs VS Code extension.

## Integration Layers

| Layer | Status | Notes |
|-------|--------|-------|
| **MCP Context Server** | ✅ Implemented | `operator mcp` registered via `context_server_command()` |
| **ACP Agent Server** | ✅ Setup command | `/op-setup-agent` generates config for `~/.config/zed/settings.json` |
| **Slash Commands** | ✅ 14 commands | All original commands + `/op-setup`, `/op-help`, `/op-setup-agent` |
| **Guided Onboarding** | ✅ `/op-setup` | Health check checklist with next-step guidance |
| **Install Instructions** | ✅ Updated | Pre-built binary downloads from GitHub releases |

## VS Code Features → Zed Status

| Feature | VS Code | Zed | Notes |
|---------|---------|-----|-------|
| **MCP Tools** | ✅ Auto-connect | ✅ Context server | Native MCP via `operator mcp` stdio |
| **ACP Agent** | N/A | ✅ Via settings | Operator as agent in Agent Panel |
| **Slash Commands** | N/A | ✅ 12 commands | Thin inference layer for humans + AI |
| **Sidebar Views** | ✅ 4 TreeViews | ❌ N/A | MCP tools serve as alternative |
| **Status Bar** | ✅ Live indicator | ❌ N/A | Use `/op-status` or MCP health tool |
| **Webhook Server** | ✅ Port 7009 | ❌ N/A | MCP polling instead |
| **Terminal Management** | ✅ Create/style/track | ❌ N/A | ACP agent replaces terminal sessions |
| **File Watching** | ✅ .tickets/ watcher | ❌ N/A | Manual refresh via commands/tools |
| **Guided Onboarding** | ✅ 4-step walkthrough | ✅ `/op-setup` | Health check checklist with next steps |
| **Launch Options Dialog** | ✅ Multi-select | ❌ N/A | `/op-launch` with tab completion |
| **Color-coded Terminals** | ✅ By issue type | ❌ N/A | Use Zed Tasks as workaround |
| **Binary Download** | ✅ Command | ❌ PATH discovery | Extension finds binary on PATH |

## Zed-Exclusive Capabilities

Features Zed has that VS Code doesn't:

1. **Native MCP integration** — tools appear directly in Agent Panel without manual config
2. **ACP agent sessions** — prompts flow through Operator to Claude Code delegator
3. **AI-accessible slash commands** — both humans and AI can use them in the assistant

## Not Possible in Zed (API Limitations)

1. **Sidebar views** — no TreeDataProvider equivalent
2. **Status bar items** — no extension API
3. **Terminal management** — no programmatic terminal API
4. **File watching** — no extension file watcher
5. **Agent server from WASM** — must use settings.json config (extension.toml requires binary downloads)
6. **Webhook server** — WASM sandbox prevents port listening

## Future Improvements

When Zed's extension API expands:

- [ ] Agent server registration from WASM (no manual settings.json)
- [ ] Sidebar views if TreeDataProvider added
- [ ] Status bar item if API becomes available
- [ ] File watching for auto-refresh
- [ ] Configuration UI when settings API available
