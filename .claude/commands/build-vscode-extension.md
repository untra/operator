---
description: Build, lint, test, package, and install the VS Code extension locally
allowed-tools: Bash, Read
model: sonnet
---

# Build VS Code Extension

Build, validate, package, and install the `vscode-extension` subproject for local inspection. Stop immediately on any failure and report the error.

## Workflow

Run each step sequentially from the `vscode-extension/` directory. If any step fails, stop and report the failure clearly.

1. **Install dependencies**: `cd vscode-extension && npm install`
2. **Lint**: `cd vscode-extension && npm run lint`
3. **Compile**: `cd vscode-extension && npm run compile`
4. **Test**: `cd vscode-extension && npm test` (note: requires a display environment for @vscode/test-electron; if tests fail due to missing display, report it and continue)
5. **Package**: `cd vscode-extension && npm run package` (creates a `.vsix` file via vsce)
6. **Detect version**: `cd vscode-extension && node -p "require('./package.json').version"`
7. **Install extension**: `cd vscode-extension && code --install-extension ./operator-terminals-VERSION.vsix` (substitute the detected version)
8. **Report**: Confirm the extension was installed successfully. Remind the user they must reload their VS Code window (`Developer: Reload Window` from the command palette) for changes to take effect.

## Notes

- The `npm run compile` step runs `copy-types` then `tsc`.
- The `.vsix` filename follows the pattern `operator-terminals-VERSION.vsix`.
- If `code` CLI is not on PATH, suggest the user install it via VS Code command palette: "Shell Command: Install 'code' command in PATH".
