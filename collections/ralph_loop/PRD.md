---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
project: {{ project }}
{{#if priority }}priority: {{ priority }}
{{/if}}---

# PRD: {{ summary }}

{{#if source_notes }}
## Source Notes
{{ source_notes }}
{{/if}}

{{#if quality_commands }}
## Quality Commands
{{ quality_commands }}
{{/if}}
