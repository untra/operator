---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
project: {{ project }}
feature_id: {{ feature_id }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# JR Task: {{ summary }}

## Acceptance
{{ acceptance }}

{{#if dependencies }}
## Dependencies
{{ dependencies }}
{{/if}}
