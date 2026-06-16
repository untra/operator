---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
project: {{ project }}
time_budget_hours: {{ time_budget_hours }}
---

# Elves Stage: {{ summary }}

## Run Goal
{{ run_goal }}

{{#if validation_commands }}
## Validation Commands
{{ validation_commands }}
{{/if}}
