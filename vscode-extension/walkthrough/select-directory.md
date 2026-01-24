# Select Working Directory

Choose a **parent directory** that contains (or will contain) your project repositories.

Operator will create a `.tickets/` folder here to manage work items across all projects in this directory.

## Directory Structure

```
~/code/                    <- Select this directory
  .tickets/               <- Created automatically
    queue/
    in-progress/
    completed/
  project-a/              <- Your repos
  project-b/
  project-c/
```

## What happens next

After selecting a directory:

1. Operator creates the `.tickets/` structure
2. You can start creating markdown work tickets for any project
3. LLM agents will work the tickets and track progress

Click **Select Directory** above to get started.
