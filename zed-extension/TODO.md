# Zed Extension TODO

Feature comparison and implementation status vs VS Code extension.

## VS Code Extension Commands → Zed Slash Commands

| VS Code Command | Zed Equivalent | Status |
|-----------------|----------------|--------|
| `operator.showStatus` | `/op-status` | ✅ Implemented |
| `operator.refreshTickets` | N/A | ❌ No UI to refresh |
| `operator.focusTicket` | N/A | ❌ No terminal API |
| `operator.openTicket` | `/op-ticket` | ✅ Shows details (can't open file) |
| `operator.launchTicket` | `/op-launch` | ✅ Implemented |
| `operator.launchTicketWithOptions` | N/A | ❌ No dialog API |
| `operator.relaunchTicket` | `/op-launch` | ✅ Can relaunch same ticket |
| `operator.launchTicketFromEditor` | N/A | ❌ No editor context |
| `operator.downloadOperator` | N/A | ❌ Use manual install |
| `operator.pauseQueue` | `/op-pause` | ✅ Implemented |
| `operator.resumeQueue` | `/op-resume` | ✅ Implemented |
| `operator.syncKanban` | `/op-sync` | ✅ Implemented |
| `operator.approveReview` | `/op-approve` | ✅ Implemented |
| `operator.rejectReview` | `/op-reject` | ✅ Implemented |
| `operator.startOperatorServer` | N/A | ❌ Use Tasks or terminal |

## VS Code Features → Zed Status

| Feature | VS Code | Zed | Notes |
|---------|---------|-----|-------|
| **Sidebar Views** | ✅ 4 TreeViews | ❌ N/A | Zed has no sidebar extension API |
| **Status Bar** | ✅ Live indicator | ❌ N/A | Zed has no status bar API |
| **Webhook Server** | ✅ Port 7009 | ❌ N/A | WASM sandbox prevents servers |
| **Terminal Management** | ✅ Create/style/track | ❌ N/A | Zed has no terminal extension API |
| **File Watching** | ✅ .tickets/ watcher | ❌ N/A | No file watcher in extensions |
| **REST Client** | ✅ Native fetch | ✅ curl subprocess | Works but slower |
| **Ticket Completion** | ✅ QuickPick | ✅ Argument completion | Works for ticket IDs |
| **Launch Options Dialog** | ✅ Multi-select | ❌ N/A | No dialog API |
| **Color-coded Terminals** | ✅ By issue type | ❌ N/A | Use Zed Tasks instead |

## Implemented Slash Commands

- [x] `/op-status` - Show Operator health/status
- [x] `/op-queue` - List tickets in queue
- [x] `/op-launch TICKET-ID` - Launch a ticket
- [x] `/op-active` - List active agents
- [x] `/op-completed` - List completed tickets
- [x] `/op-ticket TICKET-ID` - Show ticket details
- [x] `/op-pause` - Pause queue processing
- [x] `/op-resume` - Resume queue processing
- [x] `/op-sync` - Sync kanban collections
- [x] `/op-approve AGENT-ID` - Approve review
- [x] `/op-reject AGENT-ID REASON` - Reject review

## Not Possible in Zed

These features cannot be implemented due to Zed's extension API limitations:

1. **Sidebar Views**
   - No TreeDataProvider equivalent
   - Status, queue, active, completed views all unavailable
   - Workaround: Use slash commands to query data

2. **Webhook Server**
   - WASM sandbox prevents opening ports
   - Cannot receive notifications from Operator
   - Workaround: Poll with slash commands

3. **Terminal Management**
   - Cannot create/manage terminals programmatically
   - Cannot set terminal colors or icons
   - Workaround: Use Zed Tasks (`.zed/tasks.json`)

4. **Status Bar**
   - No API to add status bar items
   - Cannot show persistent status indicator
   - Workaround: Use `/op-status` command

5. **File System Watching**
   - Cannot watch for ticket file changes
   - Auto-refresh not possible
   - Workaround: Manual refresh via commands

6. **Editor Context Commands**
   - Cannot detect active editor file
   - Cannot launch ticket from open file
   - Workaround: Use `/op-launch` with explicit ID

## Future Improvements

When Zed's extension API expands:

- [ ] Add sidebar views if TreeDataProvider added
- [ ] Add status bar item if API becomes available
- [ ] Add terminal creation if API becomes available
- [ ] Add file watching for auto-refresh
- [ ] Add configuration UI when settings API available

## Alternative Workflows

### Using Zed Tasks

For terminal-based workflows, create `.zed/tasks.json`:

```json
[
  {
    "label": "Operator: Start API Server",
    "command": "operator api",
    "use_new_terminal": true,
    "allow_concurrent_runs": false
  },
  {
    "label": "Operator: Show Queue (CLI)",
    "command": "operator queue",
    "use_new_terminal": false
  },
  {
    "label": "Operator: Launch Next Ticket",
    "command": "operator launch --next",
    "use_new_terminal": true
  },
  {
    "label": "Operator: Show Active Agents",
    "command": "operator agents",
    "use_new_terminal": false
  }
]
```

### Using the AI Assistant

The slash commands are designed to work well in the AI assistant context:

1. Ask about status: `/op-status`
2. See what needs work: `/op-queue`
3. Get details: `/op-ticket FIX-123`
4. Launch work: `/op-launch FIX-123`
5. Monitor progress: `/op-active`

The AI assistant can use this information contextually to help with your work.
