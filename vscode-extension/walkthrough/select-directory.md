# Select Working Directory

Choose a **parent directory** that contains (or will contain) your project repositories. This is where the Operator server runs from.

Selecting a directory runs `operator setup`, which writes a `config.toml` with server settings and creates the `.tickets/` structure for managing work items.

The extension persists this path in your VS Code user settings so it works across all workspaces.

## Directory Structure

```
~/code/                    <- Select this directory
  .tickets/               <- Created by operator setup
    queue/
    in-progress/
    completed/
  config.toml             <- Server configuration
  project-a/              <- Your repos
  project-b/
  project-c/
```

## What happens next

After selecting a directory:

1. Operator runs `setup` to create the `.tickets/` structure and `config.toml`
2. The path is saved to your VS Code settings for cross-workspace access
3. You can start creating markdown work tickets for any project
4. LLM agents will work the tickets and track progress

Click **Select Directory** above to get started.
