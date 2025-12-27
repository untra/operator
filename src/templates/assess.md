---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
{{#if kind_override }}kind_override: {{ kind_override }}
{{/if}}---

# Assessment: {{ summary }}

{{#if kind_override }}
## Kind Override
{{ kind_override }}
{{/if}}
