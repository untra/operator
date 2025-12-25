---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}scope: {{ scope }}
{{#if project }}project: {{ project }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
---

# Catalog Sync: {{ summary }}

## Scope
{{ scope }}

{{#if project }}
## Target Project
{{ project }}
{{/if}}
