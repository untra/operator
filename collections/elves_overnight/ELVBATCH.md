---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
project: {{ project }}
continue_after_batch: {{ continue_after_batch }}
---

# Elves Batch: {{ summary }}

## Batch Goal
{{ batch_goal }}

{{#if validation_commands }}
## Validation Commands
{{ validation_commands }}
{{/if}}
