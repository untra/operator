---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
created: {{ created_datetime }}
project: {{ project }}
workflow_id: {{ workflow_id }}
{{#if story_id }}story_id: {{ story_id }}
{{/if}}---

# Ralph Story: {{ summary }}

{{#if notes }}
## Notes
{{ notes }}
{{/if}}
