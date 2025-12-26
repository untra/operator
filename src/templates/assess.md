---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}project: {{ project }}
status: {{ status }}
created: {{ created_datetime }}
{{#if kind_override }}kind_override: {{ kind_override }}
{{/if}}---

# Assessment: {{ summary }}

## Project
{{ project }}

{{#if kind_override }}
## Kind Override
{{ kind_override }}
{{/if}}
