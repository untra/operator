---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
project: {{ project }}
target_ref: {{ target_ref }}
---

# JR Review: {{ summary }}

{{#if review_focus }}
## Review Focus
{{ review_focus }}
{{/if}}
