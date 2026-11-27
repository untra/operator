# Runtime scripts

Keep multi-step runtime scripts here as standalone source files. Rust embeds them with `include_str!`. The VS Code extension embeds its PowerShell scripts during `npm run copy-types`, which runs before compilation and packaging.

Shell files declare their interpreter in the shebang. Templates use `__OP_NAME__` placeholders. The Rust caller supplies shell-quoted data or, for command and setup placeholders, explicitly generated shell text. Keep credentials in stdin, environment variables, or private runtime files. Never substitute values into PowerShell source.

Run `make lint-shell` for shell lint and syntax checks. On Windows, run `powershell.exe -NoProfile -File scripts/ci/test-runtime-powershell.ps1` with PSScriptAnalyzer 1.24.0 installed.
