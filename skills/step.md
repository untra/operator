---
name: operator:step
description: Signal step completion and transition to the next step
---

# Step Transition

When you have completed all work for the current step, use this skill to signal completion.

## How to signal step completion

Create or update the file `.operator/step-complete.json` with the following content:

```json
{
  "status": "complete",
  "summary": "<brief summary of what was accomplished>"
}
```

Then **stop and wait** — the operator will detect completion and provide the next step's prompt.

## Important

- Do NOT proceed to the next step on your own
- Do NOT guess what the next step should be
- Write a clear, concise summary of what you accomplished
- Ensure all work for the current step is committed or saved before signaling
